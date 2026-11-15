//! Offline QR code generator: text in, PNG out, optional viewer page.

mod cli;
mod png;
mod util;
mod viewer;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use qrcodegen::{QrCode, QrCodeEcc, QrSegment, Version};

fn main() -> ExitCode {
    let prog = util::prog_name();
    let argv = util::args();
    match cli::parse(&argv) {
        cli::Outcome::Help => {
            print!("{}", cli::help(&prog));
            ExitCode::SUCCESS
        }
        cli::Outcome::Version => {
            println!("{}", cli::VERSION);
            ExitCode::SUCCESS
        }
        cli::Outcome::Usage(message) => {
            eprint!("{}", cli::error(&prog, &message));
            ExitCode::from(2)
        }
        cli::Outcome::Run(args) => ExitCode::from(run(args, &prog)),
    }
}

fn run(args: cli::Args, prog: &str) -> u8 {
    let cli::Args {
        text,
        file,
        out,
        scale,
        border,
        html,
        title,
    } = args;

    let data = match file.as_deref() {
        Some(path) => match util::read_stripped(path) {
            Ok(data) => data,
            Err(err) => return fail("read", path, &err),
        },
        None => text.unwrap_or_default(),
    };
    if data.is_empty() {
        eprint!("{}", cli::error(prog, "empty input"));
        return 2;
    }

    let out = util::absolute(&out);
    let page = if html {
        Some(util::page_path(&out))
    } else {
        None
    };
    if let Some(parent) = Path::new(&out).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(err) = fs::create_dir_all(parent) {
                return fail("write", &out, &err);
            }
        }
    }

    let code = match encode(&data) {
        Ok(code) => code,
        Err(()) => {
            eprintln!("FAIL store: data does not fit at error correction level M");
            return 1;
        }
    };
    let side = (code.size() as u64 + 2 * border as u64) * scale as u64;
    let side = side as u32;

    let write_png = || -> std::io::Result<()> {
        let file = fs::File::create(&out)?;
        let mut writer = std::io::BufWriter::with_capacity(128 * 1024, file);
        png::write_png(&mut writer, &code, scale, border)?;
        writer.flush()
    };
    if let Err(err) = write_png() {
        let _ = fs::remove_file(&out);
        return fail("write", &out, &err);
    }

    let (got, complete) = match png::probe(Path::new(&out)) {
        Ok(probe) => (probe.size, probe.iend),
        Err(_) => (None, false),
    };
    if got != Some((side, side)) {
        let _ = fs::remove_file(&out);
        let shown = match got {
            Some((width, height)) => format!("({}, {})", width, height),
            None => "None".to_string(),
        };
        eprintln!("FAIL png size {} expected ({}, {})", shown, side, side);
        return 1;
    }
    if !complete {
        let _ = fs::remove_file(&out);
        eprintln!("FAIL png tail: missing IEND");
        return 1;
    }

    if let Some(page) = &page {
        let name = util::base_name(&out);
        let rendered = viewer::render(&title, &name);
        let write_page = || -> std::io::Result<()> {
            let mut file = fs::File::create(page)?;
            file.write_all(rendered.as_bytes())?;
            file.flush()
        };
        if let Err(err) = write_page() {
            return fail("write", page, &err);
        }
    }

    println!("{}", out);
    if let Some(page) = page {
        println!("{}", page);
    }
    0
}

fn encode(data: &str) -> Result<QrCode, ()> {
    let segment = if QrSegment::is_numeric(data) {
        QrSegment::make_numeric(data)
    } else if QrSegment::is_alphanumeric(data) {
        QrSegment::make_alphanumeric(data)
    } else {
        QrSegment::make_bytes(data.as_bytes())
    };
    let segments = [segment];
    QrCode::encode_segments_advanced(
        &segments,
        QrCodeEcc::Medium,
        Version::MIN,
        Version::MAX,
        None,
        false,
    )
    .map_err(|_| ())
}

fn fail(kind: &str, path: &str, err: &std::io::Error) -> u8 {
    eprintln!("FAIL {}: {}: {}", kind, path, err);
    1
}
