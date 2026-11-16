//! Process tests for the built binary: the save path, the refusals and the
//! cleanup a run has to leave behind. The decode checks live in tools/gate.py;
//! these run with `cargo test` on every build host and need no Python.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_qr");

struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create the scratch directory");
        Scratch { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn run(&self, args: &[&str]) -> Output {
        let args: Vec<&OsStr> = args.iter().map(|arg| OsStr::new(*arg)).collect();
        Command::new(BIN)
            .args(&args)
            .current_dir(&self.dir)
            .output()
            .expect("run the binary")
    }

    fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.dir).expect("read the scratch directory") {
            let name = entry.expect("directory entry").file_name();
            names.push(name.to_string_lossy().into_owned());
        }
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn save_writes_one_file_and_prints_its_absolute_path() {
    let scratch = Scratch::new("save-one");
    let output = scratch.run(&["x", "-o", "out.png"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let printed = stdout(&output);
    assert_eq!(printed.lines().count(), 1, "{}", printed);
    let line = printed.trim_end();
    assert!(Path::new(line).is_absolute(), "{}", line);
    let resolved = fs::canonicalize(line).expect("resolve the printed path");
    let want = fs::canonicalize(scratch.path("out.png")).expect("resolve the output");
    assert_eq!(resolved, want);
    assert_eq!(scratch.names(), vec!["out.png".to_string()]);
    let bytes = fs::read(scratch.path("out.png")).expect("read the png");
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "not a png: {} bytes", bytes.len());
}

#[test]
fn terminal_mode_writes_nothing() {
    let scratch = Scratch::new("terminal");
    let output = scratch.run(&["x"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stdout(&output).is_empty(), "no painted symbol");
    assert!(scratch.names().is_empty(), "{:?}", scratch.names());
}

#[test]
fn a_page_equal_to_the_png_is_refused() {
    let scratch = Scratch::new("page-clash");
    let output = scratch.run(&["x", "-o", "out.html", "--html"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(stderr(&output).contains("page path equals the PNG path"), "{}", stderr(&output));
    assert!(scratch.names().is_empty(), "{:?}", scratch.names());
}

#[test]
fn a_read_only_destination_is_refused_and_kept() {
    let scratch = Scratch::new("read-only");
    let path = scratch.path("out.png");
    fs::write(&path, b"old").expect("seed the destination");
    let mut permissions = fs::metadata(&path).expect("read the mode").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).expect("make it read-only");

    let output = scratch.run(&["x", "-o", "out.png"]);

    let mut permissions = fs::metadata(&path).expect("read the mode").permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&path, permissions).expect("make it writable again");

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.starts_with("FAIL write:"), "{}", err);
    assert!(err.contains("destination is read-only"), "{}", err);
    assert_eq!(fs::read(&path).expect("read the old bytes"), b"old");
    assert_eq!(scratch.names(), vec!["out.png".to_string()]);
}

#[test]
fn a_blocked_page_keeps_the_png_and_leaves_no_temp() {
    let scratch = Scratch::new("page-blocked");
    fs::create_dir(scratch.path("out.html")).expect("block the page path");
    let output = scratch.run(&["x", "-o", "out.png", "--html"]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.starts_with("FAIL write:"), "{}", err);
    assert!(scratch.path("out.png").is_file(), "the png is gone");
    assert!(scratch.path("out.html").is_dir(), "the blocker is gone");
    let temp: Vec<String> = scratch
        .names()
        .into_iter()
        .filter(|name| name.contains(".tmp"))
        .collect();
    assert!(temp.is_empty(), "{:?}", temp);
}

#[test]
fn an_oversize_payload_creates_no_directory() {
    let scratch = Scratch::new("oversize");
    let payload = "x".repeat(3000);
    let output = scratch.run(&[payload.as_str(), "-o", "deep/out.png"]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stderr(&output).starts_with("FAIL store:"), "{}", stderr(&output));
    assert!(scratch.names().is_empty(), "{:?}", scratch.names());
}

#[test]
fn the_page_is_written_beside_the_png() {
    let scratch = Scratch::new("page");
    let output = scratch.run(&["x", "-o", "out.png", "--html"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let printed = stdout(&output);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines.len(), 2, "{}", printed);
    let resolved: Vec<PathBuf> = lines
        .iter()
        .map(|line| fs::canonicalize(line).expect("resolve a printed path"))
        .collect();
    let want = vec![
        fs::canonicalize(scratch.path("out.png")).expect("resolve the png"),
        fs::canonicalize(scratch.path("out.html")).expect("resolve the page"),
    ];
    assert_eq!(resolved, want);
    assert_eq!(scratch.names(), vec!["out.html".to_string(), "out.png".to_string()]);
    let page = fs::read_to_string(scratch.path("out.html")).expect("read the page");
    assert!(page.contains("src=\"out.png\""), "{}", page);
}

#[test]
fn a_missing_input_is_one_fail_line() {
    let scratch = Scratch::new("missing");
    let output = scratch.run(&["-f", "nope.txt"]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.starts_with("FAIL read:"), "{}", err);
    assert_eq!(err.lines().count(), 1, "{}", err);
    assert!(stdout(&output).is_empty(), "{}", stdout(&output));
    assert!(scratch.names().is_empty(), "{:?}", scratch.names());
}
