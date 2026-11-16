//! Terminal display: the symbol printed with half blocks, no file access.
//!
//! Two module rows share one text row, so a module stays square in a cell
//! that is twice as tall as it is wide. The quiet zone is painted with the
//! page, and the background is forced light, because a dark terminal would
//! otherwise present an inverted symbol that many scanners refuse.

use qrcodegen::QrCode;

const INK: &str = "\x1b[30;107m";
const RESET: &str = "\x1b[0m";
const FULL: char = '\u{2588}';
const UPPER: char = '\u{2580}';
const LOWER: char = '\u{2584}';

/// Render the whole symbol, quiet zone included, as printable text.
pub fn render(code: &QrCode, border: u32) -> String {
    let border = border as i32;
    let span = code.size() + 2 * border;
    let rows = (span + 1) / 2;
    // A dense symbol paints a three-byte glyph in every cell.
    let capacity =
        span as usize * rows as usize * 3 + rows as usize * (INK.len() + RESET.len() + 1);
    let mut out = String::with_capacity(capacity);
    let mut y = -border;
    while y < -border + span {
        out.push_str(INK);
        for x in -border..-border + span {
            let upper = dark(code, x, y);
            let lower = dark(code, x, y + 1);
            out.push(match (upper, lower) {
                (true, true) => FULL,
                (true, false) => UPPER,
                (false, true) => LOWER,
                (false, false) => ' ',
            });
        }
        out.push_str(RESET);
        out.push('\n');
        y += 2;
    }
    out
}

fn dark(code: &QrCode, x: i32, y: i32) -> bool {
    let size = code.size();
    x >= 0 && y >= 0 && x < size && y < size && code.get_module(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcodegen::QrCodeEcc;

    fn sample() -> QrCode {
        QrCode::encode_text("ab", QrCodeEcc::Medium).unwrap()
    }

    fn body(line: &str) -> &str {
        line.strip_prefix(INK)
            .and_then(|rest| rest.strip_suffix(RESET))
            .expect("a painted line")
    }

    #[test]
    fn every_line_is_painted_and_wrapped() {
        let code = sample();
        let text = render(&code, 4);
        let lines: Vec<&str> = text.split_terminator('\n').collect();
        let span = code.size() + 8;
        assert_eq!(lines.len() as i32, (span + 1) / 2);
        for line in lines {
            assert_eq!(body(line).chars().count() as i32, span);
        }
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn the_grid_round_trips_to_the_modules() {
        let code = sample();
        let border = 4;
        let span = code.size() + 2 * border;
        let text = render(&code, border as u32);
        let mut grid = vec![vec![false; span as usize]; span as usize];
        for (row, line) in text.split_terminator('\n').enumerate() {
            for (x, cell) in body(line).chars().enumerate() {
                let (upper, lower) = match cell {
                    FULL => (true, true),
                    UPPER => (true, false),
                    LOWER => (false, true),
                    ' ' => (false, false),
                    other => panic!("unexpected glyph {:?}", other),
                };
                let y = row as i32 * 2;
                if upper && y < span {
                    grid[y as usize][x] = true;
                }
                if lower && y + 1 < span {
                    grid[(y + 1) as usize][x] = true;
                }
            }
        }
        for y in 0..span {
            for x in 0..span {
                let inside = x >= border
                    && y >= border
                    && x < border + code.size()
                    && y < border + code.size();
                let want = inside && code.get_module(x - border, y - border);
                assert_eq!(grid[y as usize][x as usize], want, "module ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn the_quiet_zone_is_blank_on_every_side() {
        let code = sample();
        let text = render(&code, 2);
        let lines: Vec<&str> = text.split_terminator('\n').collect();
        for line in lines.iter().take(1) {
            assert!(body(line).chars().all(|cell| cell == ' '));
        }
        for line in lines.iter() {
            let row: Vec<char> = body(line).chars().collect();
            assert_eq!(&row[..2], &[' ', ' ']);
            assert_eq!(&row[row.len() - 2..], &[' ', ' ']);
        }
    }
}
