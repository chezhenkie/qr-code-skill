//! Offline QR code generator: text in, PNG out or terminal display, optional viewer page.

mod cli;
mod png;
mod term;
mod util;
mod viewer;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use qrcodegen::{QrCode, QrCodeEcc, QrSegment, Version};

fn main() -> ExitCode {
    let prog = util::prog_name();
    let argv = util::args();
    match cli::parse(&argv) {
        cli::Outcome::Help => match write_stdout(&cli::help(&prog)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => ExitCode::from(fail("write", "stdout", &err)),
        },
        cli::Outcome::Version => match write_stdout(&format!("{}\n", cli::VERSION)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => ExitCode::from(fail("write", "stdout", &err)),
        },
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
        save,
    } = args;

    let data = match &file {
        Some(path) => match util::read_stripped(path) {
            Ok(data) => data,
            Err(err) => return fail("read", &util::display(path), &err),
        },
        None => text.unwrap_or_default(),
    };
    if data.is_empty() {
        eprint!("{}", cli::error(prog, "empty input"));
        return 2;
    }

    if !save {
        let code = match encode_or_fail(&data) {
            Ok(code) => code,
            Err(code) => return code,
        };
        let painted = term::render(&code, border);
        return match write_stdout(&painted) {
            Ok(()) => 0,
            Err(err) => fail("write", "stdout", &err),
        };
    }

    let (out, page) = match util::output_paths(&out, html) {
        Ok(pair) => pair,
        Err(clash) => {
            eprint!(
                "{}",
                cli::error(
                    prog,
                    &format!("page path equals the PNG path: {}", util::display(&clash))
                )
            );
            return 2;
        }
    };
    let shown = util::display(&out);

    let code = match encode_or_fail(&data) {
        Ok(code) => code,
        Err(code) => return code,
    };
    let side = (code.size() as u32 + 2 * border) * scale;

    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(err) = fs::create_dir_all(parent) {
                return fail("write", &shown, &err);
            }
        }
    }
    if let Err(err) = refuse_readonly(&out) {
        return fail("write", &shown, &err);
    }

    // The image goes to a temp name beside the final one and is renamed only
    // after the header check, so a failed run never truncates an existing
    // image and no reader sees a partial file.
    let temp = util::temp_path(&out);
    if let Err(err) = write_png_file(&temp, &code, scale, border) {
        let _ = fs::remove_file(&temp);
        return fail("write", &shown, &err);
    }
    let probe = match png::probe(&temp) {
        Ok(probe) => probe,
        Err(_) => png::Probe {
            size: None,
            iend: false,
        },
    };
    if let Err(line) = png::verify(&probe, side) {
        let _ = fs::remove_file(&temp);
        eprintln!("{}", line);
        return 1;
    }
    if let Err(err) = fs::rename(&temp, &out) {
        let _ = fs::remove_file(&temp);
        return fail("write", &shown, &err);
    }

    if let Some(page) = &page {
        let name = util::page_name(&out);
        let rendered = viewer::render(&title, &name);
        let shown_page = util::display(page);
        if let Err(err) = refuse_readonly(page) {
            return fail("write", &shown_page, &err);
        }
        let temp = util::temp_path(page);
        if let Err(err) = write_page_file(&temp, rendered.as_bytes()) {
            let _ = fs::remove_file(&temp);
            return fail("write", &shown_page, &err);
        }
        if let Err(err) = fs::rename(&temp, page) {
            let _ = fs::remove_file(&temp);
            return fail("write", &shown_page, &err);
        }
    }

    let mut printed = write_stdout(&format!("{}\n", shown));
    if printed.is_ok() {
        if let Some(page) = &page {
            printed = write_stdout(&format!("{}\n", util::display(page)));
        }
    }
    if let Err(err) = printed {
        return fail("write", "stdout", &err);
    }
    0
}

/// Refuse to replace a file that is read-only, on every platform. Unix would
/// let the rename replace it, Windows would not; one rule both can rely on.
/// The path itself is inspected, so replacing a symlink is judged by the link.
fn refuse_readonly(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && meta.permissions().readonly() => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "destination is read-only",
        )),
        _ => Ok(()),
    }
}

fn write_png_file(path: &Path, code: &QrCode, scale: u32, border: u32) -> io::Result<()> {
    let file = fs::File::create(path)?;
    let mut writer = io::BufWriter::with_capacity(128 * 1024, file);
    png::write_png(&mut writer, code, scale, border)?;
    writer.flush()
}

fn write_page_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.flush()
}

/// One checked write to stdout: a closed pipe is a FAIL line, not a panic.
fn write_stdout(text: &str) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    stdout.write_all(text.as_bytes())?;
    stdout.flush()
}

fn encode_or_fail(data: &str) -> Result<QrCode, u8> {
    match encode(data) {
        Ok(code) => Ok(code),
        Err(()) => {
            eprintln!("FAIL store: data does not fit at error correction level M");
            Err(1)
        }
    }
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
