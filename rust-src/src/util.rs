//! Paths, argument capture and the CPython strip set.

use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

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

/// The absolute form of a path, with `.` and `..` resolved lexically, like os.path.abspath.
pub fn absolute(path: &str) -> String {
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
        path.rfind(|c| c == '/' || c == '\\')
    } else {
        path.rfind('/')
    }
}

/// Read a file as UTF-8 with line endings kept, then strip CPython whitespace.
pub fn read_stripped(path: &str) -> io::Result<String> {
    let bytes = fs::read(path)?;
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
        assert_eq!(page_path("..."), "...html");
        assert_eq!(page_path("dir.d/qr"), "dir.d/qr.html");
    }

    #[test]
    fn base_name_is_the_last_component() {
        assert_eq!(base_name("a/b/qr.png"), "qr.png");
        assert_eq!(base_name("/qr.png"), "qr.png");
    }
}
