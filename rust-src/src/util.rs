//! Paths, argument capture and the CPython strip set.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// The largest input file in bytes. Far above the level-M payload maximum,
/// so any payload that could fit passes; the bound only stops a special file
/// or an accidental huge read.
pub const MAX_INPUT: u64 = 1024 * 1024;

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

/// Arguments after the program name, decoded as lossy UTF-8.
pub fn args() -> Vec<String> {
    env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// The absolute form of a path. The OS resolves it where it can, which is
/// what a Windows drive-relative name such as C:drive.png needs.
pub fn absolute(path: &str) -> String {
    if let Ok(resolved) = std::path::absolute(path) {
        return resolved.to_string_lossy().into_owned();
    }
    lexical_absolute(path)
}

/// Component walk used when the OS refuses the path, as for an empty name.
fn lexical_absolute(path: &str) -> String {
    let given = Path::new(path);
    let mut out = if given.is_absolute() {
        PathBuf::new()
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    for component in given.components() {
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
    out.to_string_lossy().into_owned()
}

/// The absolute PNG path and, with a page, the page path. A page that lands
/// on the PNG itself is refused: it would truncate the image just written.
pub fn output_paths(out: &str, html: bool) -> Result<(String, Option<String>), String> {
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

/// Whether two names land on the same file under this platform's rule.
fn same_file(a: &str, b: &str) -> bool {
    if cfg!(any(windows, target_os = "macos")) {
        a.to_lowercase() == b.to_lowercase()
    } else {
        a == b
    }
}

/// The page path for a PNG path: the same name with the extension replaced by .html.
pub fn page_path(png_path: &str) -> String {
    let (stem, _) = split_extension(png_path);
    let mut page = String::with_capacity(stem.len() + 5);
    page.push_str(stem);
    page.push_str(".html");
    page
}

/// The last path component, for the page's image reference.
pub fn base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Split a path at its extension, following os.path.splitext.
fn split_extension(path: &str) -> (&str, &str) {
    let start = last_separator(path).map(|index| index + 1).unwrap_or(0);
    if let Some(dot) = path.rfind('.') {
        if dot >= start && path[start..dot].chars().any(|c| c != '.') {
            return (&path[..dot], &path[dot..]);
        }
    }
    (path, "")
}

fn last_separator(path: &str) -> Option<usize> {
    if cfg!(windows) {
        path.rfind(['/', '\\'])
    } else {
        path.rfind('/')
    }
}

/// Read a file as UTF-8 with line endings kept, then strip CPython whitespace.
pub fn read_stripped(path: &str) -> io::Result<String> {
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
        assert_eq!(page_path("qr.png"), "qr.html");
        assert_eq!(page_path("a.b.c"), "a.b.html");
        assert_eq!(page_path("qr"), "qr.html");
        assert_eq!(page_path(".png"), ".png.html");
        assert_eq!(page_path("..."), "....html");
        assert_eq!(page_path("dir.d/qr"), "dir.d/qr.html");
    }

    #[test]
    fn base_name_is_the_last_component() {
        assert_eq!(base_name("a/b/qr.png"), "qr.png");
        assert_eq!(base_name("/qr.png"), "qr.png");
    }

    #[test]
    fn output_paths_pair_the_page_and_refuse_a_clash() {
        let (png, page) = output_paths("out.png", true).expect("a page");
        assert!(png.ends_with("out.png"), "{}", png);
        let want = page_path(&png);
        assert_eq!(page.as_deref(), Some(want.as_str()));
        assert!(output_paths("out.html", true).is_err());
        assert!(output_paths("out.html", false).is_ok());
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn a_page_clash_ignores_case_on_this_platform() {
        assert!(output_paths("OUT.HTML", true).is_err());
    }

    #[test]
    fn absolute_returns_an_os_resolved_path() {
        let got = absolute("sub/../x.png");
        assert!(Path::new(&got).is_absolute(), "{}", got);
        assert!(got.ends_with("x.png"), "{}", got);
    }

    #[cfg(windows)]
    #[test]
    fn a_drive_relative_output_resolves_to_an_absolute_path() {
        let got = absolute("C:drive.png");
        assert!(Path::new(&got).is_absolute(), "{}", got);
    }
}
