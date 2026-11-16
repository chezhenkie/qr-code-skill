//! Prints the version and module count segno would choose for each corpus row.

use std::io::Read;

use qrcodegen::{QrCode, QrCodeEcc, QrSegment, Version};

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("bad hex digit"),
    }
}

fn unhex(line: &str) -> Vec<u8> {
    let bytes = line.as_bytes();
    assert!(bytes.len().is_multiple_of(2), "odd hex length");
    (0..bytes.len() / 2)
        .map(|i| (hex_value(bytes[2 * i]) << 4) | hex_value(bytes[2 * i + 1]))
        .collect()
}

fn segment(text: &str) -> QrSegment {
    if QrSegment::is_numeric(text) {
        QrSegment::make_numeric(text)
    } else if QrSegment::is_alphanumeric(text) {
        QrSegment::make_alphanumeric(text)
    } else {
        QrSegment::make_bytes(text.as_bytes())
    }
}

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read stdin");
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let text = String::from_utf8(unhex(line)).expect("utf-8");
        let segs = [segment(&text)];
        let code = QrCode::encode_segments_advanced(
            &segs,
            QrCodeEcc::Medium,
            Version::MIN,
            Version::MAX,
            None,
            false,
        )
        .expect("payload fits");
        let size = code.size();
        println!("{}\t{}\t{}", (size - 17) / 4, size, line);
    }
}
