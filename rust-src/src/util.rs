//! Paths, argument capture and the CPython strip set.
//!
//! Every path travels as a `Path`/`OsString`, so a name that is not valid
//! Unicode keeps its bytes. Lossy decoding happens only where a value is
//! printed for a person to read.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// The largest input file in bytes. Far above the level-M payload maximum,
/// so any payload that could fit passes; the bound only stops a special file
/// or an accidental huge read.
pub const MAX_INPUT: u64 = 1024 * 1024;

/// Native path units: bytes on Unix, UTF-16 code units on Windows. Keeping the
/// platform units lets a name that is not valid Unicode round-trip exactly.
#[cfg(unix)]
mod units {
    use std::ffi::{OsStr, OsString};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    pub type Unit = u8;

    pub fn of(text: &OsStr) -> Vec<Unit> {
        text.as_bytes().to_vec()
    }

    pub fn build(parts: &[Unit]) -> OsString {
        OsString::from_vec(parts.to_vec())
    }
}

#[cfg(windows)]
mod units {
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    pub type Unit = u16;

    pub fn of(text: &OsStr) -> Vec<Unit> {
        text.encode_wide().collect()
    }

    pub fn build(parts: &[Unit]) -> OsString {
        OsString::from_wide(parts)
    }
}

const DOT: units::Unit = b'.' as units::Unit;

fn is_separator(unit: units::Unit) -> bool {
    if unit == b'/' as units::Unit {
        return true;
    }
    cfg!(windows) && unit == b'\\' as units::Unit
}

/// The program name argparse would print in usage lines: the base name of argv[0].
pub fn prog_name() -> String {
    let first = env::args_os().next();
    first
        .as_deref()
        .and_then(|arg| Path::new(arg).file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "qr".to_string())
}

/// Arguments after the program name, exactly as the operating system gave them.
pub fn args() -> Vec<OsString> {
    env::args_os().skip(1).collect()
}

/// A path as it appears in a message: a unit that is not valid Unicode shows
/// as the replacement character, while the file operation uses the exact name.
pub fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Strip an ASCII prefix from a raw argument and keep the rest exactly.
pub fn strip_ascii_prefix(text: &OsStr, prefix: &str) -> Option<OsString> {
    let all = units::of(text);
    let head: Vec<units::Unit> = prefix
        .as_bytes()
        .iter()
        .map(|&byte| byte as units::Unit)
        .collect();
    if all.starts_with(&head) {
        Some(units::build(&all[head.len()..]))
    } else {
        None
    }
}

/// The absolute form of a path. The OS resolves it where it can, which is
/// what a Windows drive-relative name such as C:drive.png needs.
pub fn absolute(path: &Path) -> PathBuf {
    match std::path::absolute(path) {
        Ok(resolved) => resolved,
        Err(_) => lexical_absolute(path),
    }
}

/// Component walk used when the OS refuses the path, as for an empty name.
fn lexical_absolute(path: &Path) -> PathBuf {
    let mut out = if path.is_absolute() {
        PathBuf::new()
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(Component::RootDir.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(part) => out.push(part),
        }
    }
    out
}

/// The absolute PNG path and, with a page, the page path. A page that lands
/// on the PNG itself is refused: it would truncate the image just written.
pub fn output_paths(out: &Path, html: bool) -> Result<(PathBuf, Option<PathBuf>), PathBuf> {
    let png = absolute(out);
    if !html {
        return Ok((png, None));
    }
    let page = page_path(&png);
    if same_file(&png, &page) {
        return Err(png);
    }
    Ok((png, Some(page)))
}

/// Whether two names land on the same file under this platform's rule. The
/// comparison decodes lossily; both names come from the same input, so the
/// only units that can differ are the ASCII extension, and the page name can
/// never be mistaken for the image name in the safe direction.
fn same_file(a: &Path, b: &Path) -> bool {
    if cfg!(any(windows, target_os = "macos")) {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

/// The page path for a PNG path: the same name with the extension replaced by
/// .html, following os.path.splitext. Works on the platform units, so a name
/// that is not valid Unicode keeps its bytes.
pub fn page_path(png: &Path) -> PathBuf {
    let all = units::of(png.as_os_str());
    let start = all
        .iter()
        .rposition(|&unit| is_separator(unit))
        .map_or(0, |at| at + 1);
    let stem_len = match all.iter().rposition(|&unit| unit == DOT) {
        Some(dot) if dot >= start && all[start..dot].iter().any(|&unit| unit != DOT) => dot,
        _ => all.len(),
    };
    let mut name = units::build(&all[..stem_len]);
    name.push(".html");
    PathBuf::from(name)
}

/// The name for the page's image reference, as the bytes a browser resolves
/// against the folder. Unix keeps the exact bytes; Windows converts the
/// UTF-16 name to UTF-8, which cannot express an unpaired surrogate.
#[cfg(unix)]
pub fn page_name(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.file_name().unwrap_or_default().as_bytes().to_vec()
}

#[cfg(windows)]
pub fn page_name(path: &Path) -> Vec<u8> {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .as_bytes()
        .to_vec()
}

/// The temp file used while writing a final path: same directory, so the
/// rename stays on one file system, and a process id so two runs cannot
/// collide.
pub fn temp_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(name)
}

/// Read a file as UTF-8 with line endings kept, then strip CPython whitespace.
pub fn read_stripped(path: &Path) -> io::Result<String> {
    let mut reader = fs::File::open(path)?.take(MAX_INPUT + 1);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_INPUT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file larger than 1 MiB",
        ));
    }
    let text = String::from_utf8(bytes).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("not valid UTF-8: {}", err.utf8_error()),
        )
    })?;
    Ok(strip_cpython(&text).to_string())
}

/// Strip the 29 code points CPython's str.strip removes, not Rust's White_Space set.
fn strip_cpython(text: &str) -> &str {
    text.trim_matches(is_cpython_space)
}

fn is_cpython_space(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_the_cpython_set() {
        assert_eq!(strip_cpython("\u{1C}a\u{1F}"), "a");
        assert_eq!(strip_cpython(" \t\r\na\u{3000}\u{202F}"), "a");
        assert_eq!(strip_cpython("a\u{1C}b"), "a\u{1C}b");
        assert_eq!(strip_cpython("\u{200B}a"), "\u{200B}a");
    }

    #[test]
    fn page_path_follows_splitext() {
        let path = |text: &str| page_path(Path::new(text));
        assert_eq!(path("qr.png"), PathBuf::from("qr.html"));
        assert_eq!(path("a.b.c"), PathBuf::from("a.b.html"));
        assert_eq!(path("qr"), PathBuf::from("qr.html"));
        assert_eq!(path(".png"), PathBuf::from(".png.html"));
        assert_eq!(path("..."), PathBuf::from("....html"));
        assert_eq!(path("dir.d/qr"), PathBuf::from("dir.d/qr.html"));
        assert_eq!(path("dir.d/"), PathBuf::from("dir.d/.html"));
    }

    #[cfg(unix)]
    #[test]
    fn a_page_path_keeps_a_non_utf8_name() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"bad\xff.png");
        let page = page_path(Path::new(name));
        assert_eq!(page.as_os_str().as_bytes(), b"bad\xff.html");
        assert_eq!(page_name(Path::new(name)), b"bad\xff.png");
    }

    #[test]
    fn page_name_is_the_last_component() {
        assert_eq!(page_name(Path::new("a/b/qr.png")), b"qr.png");
        assert_eq!(page_name(Path::new("/qr.png")), b"qr.png");
    }

    #[test]
    fn output_paths_pair_the_page_and_refuse_a_clash() {
        let (png, page) = output_paths(Path::new("out.png"), true).expect("a page");
        assert!(png.ends_with("out.png"), "{}", display(&png));
        let want = page_path(&png);
        assert_eq!(page.as_deref(), Some(want.as_path()));
        assert!(output_paths(Path::new("out.html"), true).is_err());
        assert!(output_paths(Path::new("out.html"), false).is_ok());
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn a_page_clash_ignores_case_on_this_platform() {
        assert!(output_paths(Path::new("OUT.HTML"), true).is_err());
    }

    #[test]
    fn absolute_returns_an_os_resolved_path() {
        let got = absolute(Path::new("sub/../x.png"));
        assert!(got.is_absolute(), "{}", display(&got));
        assert!(got.ends_with("x.png"), "{}", display(&got));
    }

    #[cfg(windows)]
    #[test]
    fn a_drive_relative_output_resolves_to_an_absolute_path() {
        let got = absolute(Path::new("C:drive.png"));
        assert!(got.is_absolute(), "{}", display(&got));
    }

    #[test]
    fn temp_path_sits_beside_the_final_name() {
        let temp = temp_path(Path::new("dir/out.png"));
        assert_eq!(temp.parent(), Some(Path::new("dir")));
        let name = temp.file_name().unwrap_or_default().to_string_lossy();
        assert!(name.starts_with(".out.png."), "{}", name);
        assert!(name.ends_with(".tmp"), "{}", name);
    }

    #[test]
    fn strip_ascii_prefix_keeps_the_tail_exactly() {
        let value = strip_ascii_prefix(OsStr::new("--out=a.png"), "--out=").expect("prefix");
        assert_eq!(value, OsString::from("a.png"));
        assert!(strip_ascii_prefix(OsStr::new("--out=a.png"), "--in=").is_none());
    }
}
