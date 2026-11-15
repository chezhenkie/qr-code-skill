//! Viewer page written next to a PNG, for scanning from a screen.

/// Render the page. Title and image name are escaped; the payload never reaches it.
pub fn render(title: &str, png_name: &str) -> String {
    let title = escape(title);
    let png = escape(png_name);
    format!(
        concat!(
            "<!doctype html>\n",
            "<html lang=\"en\">\n",
            "<head>\n",
            "<meta charset=\"utf-8\">\n",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "<title>{title}</title>\n",
            "<style>\n",
            "  body {{ background: #fff; color: #111; font-family: Consolas, monospace; text-align: center; margin: 24px; }}\n",
            "  img {{ width: 70vmin; height: 70vmin; image-rendering: pixelated; border: 1px solid #ddd; }}\n",
            "  p {{ color: #555; font-size: 14px; }}\n",
            "</style>\n",
            "</head>\n",
            "<body>\n",
            "<img src=\"{png}\" alt=\"qr code\">\n",
            "<p>{title}</p>\n",
            "</body>\n",
            "</html>\n",
        ),
        title = title,
        png = png,
    )
}

/// html.escape with quote=True: & < > " ' become entities.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_matches_the_python_template() {
        let want = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>qr code</title>\n<style>\n  body { background: #fff; color: #111; font-family: Consolas, monospace; text-align: center; margin: 24px; }\n  img { width: 70vmin; height: 70vmin; image-rendering: pixelated; border: 1px solid #ddd; }\n  p { color: #555; font-size: 14px; }\n</style>\n</head>\n<body>\n<img src=\"qr.png\" alt=\"qr code\">\n<p>qr code</p>\n</body>\n</html>\n";
        assert_eq!(render("qr code", "qr.png"), want);
    }

    #[test]
    fn title_is_escaped_in_both_places() {
        let page = render("<b>&\"x\"</b>", "a&b.png");
        let escaped = "&lt;b&gt;&amp;&quot;x&quot;&lt;/b&gt;";
        assert_eq!(page.matches(escaped).count(), 2);
        assert!(page.contains("src=\"a&amp;b.png\""));
    }
}
