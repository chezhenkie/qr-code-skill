//! A 1-bit greyscale PNG writer for a QR raster, and the post-write probe.
//!
//! The zlib stream uses stored blocks: no compressor, no table beyond the CRCs,
//! and at one bit per pixel the overhead over a compressed stream is small.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use qrcodegen::QrCode;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const MAX_BLOCK: usize = 65535;

/// What the header check found: the IHDR size, and whether the file ends with IEND.
pub struct Probe {
    pub size: Option<(u32, u32)>,
    pub iend: bool,
}

pub fn probe(path: &Path) -> io::Result<Probe> {
    let mut file = File::open(path)?;
    probe_reader(&mut file)
}

fn probe_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Probe> {
    let length = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;
    let mut head = [0u8; 24];
    let head_len = read_full(reader, &mut head)?;
    let mut iend = false;
    if length >= 12 {
        reader.seek(SeekFrom::Start(length - 12))?;
        let mut tail = [0u8; 12];
        let tail_len = read_full(reader, &mut tail)?;
        iend = tail_len == 12 && &tail[4..8] == &b"IEND"[..];
    }
    let size = if head_len < 24 || head[..8] != SIGNATURE[..] || &head[12..16] != &b"IHDR"[..] {
        None
    } else {
        Some((be_u32(&head[16..20]), be_u32(&head[20..24])))
    };
    Ok(Probe { size, iend })
}

fn be_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_full<R: Read>(reader: &mut R, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
    Ok(filled)
}

/// Write a 1-bit greyscale PNG: dark modules black, everything else white.
pub fn write_png<W: Write>(out: &mut W, code: &QrCode, scale: u32, border: u32) -> io::Result<()> {
    let modules = code.size() as usize;
    let border = border as usize;
    let scale = scale as usize;
    let total = modules + 2 * border;
    let side = total * scale;
    let row_bytes = side.div_ceil(8);

    out.write_all(&SIGNATURE)?;
    let mut header = [0u8; 13];
    header[0..4].copy_from_slice(&(side as u32).to_be_bytes());
    header[4..8].copy_from_slice(&(side as u32).to_be_bytes());
    header[8] = 1; // bit depth
    header[9] = 0; // greyscale
    write_chunk(out, b"IHDR", &header)?;

    let mut idat = Idat::new(out);
    let mut row = Vec::with_capacity(1 + row_bytes);
    for module_row in 0..total {
        row.clear();
        row.push(0); // filter: none
        let mut acc = 0u8;
        let mut bits = 0u32;
        for module_col in 0..total {
            let inside = module_row >= border
                && module_row < modules + border
                && module_col >= border
                && module_col < modules + border;
            let dark = inside
                && code.get_module(
                    (module_col - border) as i32,
                    (module_row - border) as i32,
                );
            let pixel = if dark { 0 } else { 1 };
            for _ in 0..scale {
                acc = (acc << 1) | pixel;
                bits += 1;
                if bits == 8 {
                    row.push(acc);
                    acc = 0;
                    bits = 0;
                }
            }
        }
        if bits > 0 {
            row.push(acc << (8 - bits));
        }
        for _ in 0..scale {
            idat.write(&row)?;
        }
    }
    idat.finish()?;
    write_chunk(out, b"IEND", &[])
}

struct Idat<'a, W: Write> {
    out: &'a mut W,
    block: Vec<u8>,
    chunk: Vec<u8>,
    sum: (u32, u32),
    started: bool,
}

impl<'a, W: Write> Idat<'a, W> {
    fn new(out: &'a mut W) -> Self {
        Idat {
            out,
            block: Vec::with_capacity(MAX_BLOCK),
            chunk: Vec::with_capacity(MAX_BLOCK + 7),
            sum: (1, 0),
            started: false,
        }
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.adler_update(data);
        let mut rest = data;
        while !rest.is_empty() {
            let room = MAX_BLOCK - self.block.len();
            let take = room.min(rest.len());
            self.block.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if self.block.len() == MAX_BLOCK {
                self.emit(false)?;
            }
        }
        Ok(())
    }

    fn emit(&mut self, last: bool) -> io::Result<()> {
        self.chunk.clear();
        if !self.started {
            // zlib header: deflate, 32 KiB window, fastest level, valid check bits.
            self.chunk.extend_from_slice(&[0x78, 0x01]);
            self.started = true;
        }
        self.chunk.push(if last { 1 } else { 0 });
        let length = self.block.len() as u16;
        self.chunk.extend_from_slice(&length.to_le_bytes());
        self.chunk.extend_from_slice(&(!length).to_le_bytes());
        self.chunk.extend_from_slice(&self.block);
        self.block.clear();
        write_chunk(&mut *self.out, b"IDAT", &self.chunk)
    }

    fn finish(mut self) -> io::Result<()> {
        self.emit(true)?;
        let sum = (self.sum.1 << 16) | self.sum.0;
        write_chunk(&mut *self.out, b"IDAT", &sum.to_be_bytes())
    }

    fn adler_update(&mut self, data: &[u8]) {
        let mut low = self.sum.0;
        let mut high = self.sum.1;
        for part in data.chunks(5552) {
            for &byte in part {
                low += byte as u32;
                high += low;
            }
            low %= 65521;
            high %= 65521;
        }
        self.sum = (low, high);
    }
}

fn write_chunk<W: Write>(out: &mut W, kind: &[u8; 4], data: &[u8]) -> io::Result<()> {
    out.write_all(&(data.len() as u32).to_be_bytes())?;
    out.write_all(kind)?;
    out.write_all(data)?;
    let sum = crc32(&[&kind[..], data]);
    out.write_all(&sum.to_be_bytes())
}

fn crc32(parts: &[&[u8]]) -> u32 {
    let mut sum = 0xFFFF_FFFFu32;
    for part in parts {
        for &byte in *part {
            let index = ((sum ^ byte as u32) & 0xFF) as usize;
            sum = CRC_TABLE[index] ^ (sum >> 8);
        }
    }
    sum ^ 0xFFFF_FFFF
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = index as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 != 0 {
                0xEDB8_8320 ^ (value >> 1)
            } else {
                value >> 1
            };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
}

static CRC_TABLE: [u32; 256] = crc_table();

#[cfg(test)]
mod tests {
    use super::*;
    use qrcodegen::QrCodeEcc;
    use std::io::Cursor;

    #[test]
    fn crc_of_iend_matches_the_standard() {
        assert_eq!(crc32(&[&b"IEND"[..], &b""[..]]), 0xAE42_6082);
        assert_eq!(crc32(&[&b""[..], &b"IEND"[..]]), 0xAE42_6082);
    }

    fn chunks(bytes: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(&bytes[..8], &SIGNATURE[..]);
        let mut found = Vec::new();
        let mut pos = 8;
        while pos < bytes.len() {
            let length = be_u32(&bytes[pos..pos + 4]) as usize;
            let kind = [
                bytes[pos + 4],
                bytes[pos + 5],
                bytes[pos + 6],
                bytes[pos + 7],
            ];
            let data_start = pos + 8;
            let data = bytes[data_start..data_start + length].to_vec();
            let stored = be_u32(&bytes[data_start + length..data_start + length + 4]);
            assert_eq!(crc32(&[&kind[..], &data[..]]), stored, "chunk crc");
            found.push((kind, data));
            pos = data_start + length + 4;
        }
        found
    }

    #[test]
    fn png_has_header_idat_and_end() {
        let code = QrCode::encode_text("ab", QrCodeEcc::Medium).expect("encode");
        let mut bytes = Vec::new();
        write_png(&mut bytes, &code, 2, 4).expect("write");
        let parts = chunks(&bytes);
        assert_eq!(parts[0].0, *b"IHDR");
        assert_eq!(be_u32(&parts[0].1[0..4]), 58);
        assert_eq!(be_u32(&parts[0].1[4..8]), 58);
        assert_eq!(parts[0].1[8], 1);
        assert_eq!(parts[0].1[9], 0);
        assert_eq!(parts.last().map(|part| part.0), Some(*b"IEND"));

        let mut stream = Vec::new();
        for (kind, data) in &parts {
            if kind == b"IDAT" {
                stream.extend_from_slice(data);
            }
        }
        assert_eq!(&stream[..2], b"\x78\x01");
        let mut pos = 2;
        let mut raw = 0;
        loop {
            let header = stream[pos];
            assert_eq!(header & 0b110, 0, "stored block");
            let length = u16::from_le_bytes([stream[pos + 1], stream[pos + 2]]);
            let check = u16::from_le_bytes([stream[pos + 3], stream[pos + 4]]);
            assert_eq!(!length, check);
            raw += length as usize;
            pos += 5 + length as usize;
            if header & 1 == 1 {
                break;
            }
        }
        assert_eq!(pos + 4, stream.len(), "adler tail");
        assert_eq!(raw, 58 * (1 + 58_usize.div_ceil(8)));
    }

    #[test]
    fn probe_reads_size_and_tail() {
        let code = QrCode::encode_text("ab", QrCodeEcc::Medium).expect("encode");
        let mut bytes = Vec::new();
        write_png(&mut bytes, &code, 2, 4).expect("write");

        let mut full = Cursor::new(&bytes);
        let probe = probe_reader(&mut full).expect("probe");
        assert_eq!(probe.size, Some((58, 58)));
        assert!(probe.iend);

        let cut = &bytes[..bytes.len() - 12];
        let mut short = Cursor::new(cut);
        let probe = probe_reader(&mut short).expect("probe");
        assert_eq!(probe.size, Some((58, 58)));
        assert!(!probe.iend);
    }
}
