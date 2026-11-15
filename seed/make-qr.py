#!/usr/bin/env python3
"""Generate an offline QR code PNG, optionally with a viewer page.

No network access. Requires the segno package: python -m pip install --user segno
"""
import argparse
import html
import os

import segno

VIEWER = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  body {{ background: #fff; color: #111; font-family: Consolas, monospace; text-align: center; margin: 24px; }}
  img {{ width: 70vmin; height: 70vmin; image-rendering: pixelated; border: 1px solid #ddd; }}
  p {{ color: #555; font-size: 14px; }}
</style>
</head>
<body>
<img src="{name}" alt="qr code">
<p>{title}</p>
</body>
</html>
"""


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate an offline QR code PNG.")
    parser.add_argument("text", nargs="?", help="text to encode; omit when using --file")
    parser.add_argument("-f", "--file", help="read the text from a file, stripped")
    parser.add_argument("-o", "--out", default="qr.png", help="output PNG path")
    parser.add_argument("--scale", type=int, default=10, help="pixels per module (default 10)")
    parser.add_argument("--border", type=int, default=4, help="quiet zone in modules (default 4)")
    parser.add_argument("--html", action="store_true", help="write a viewer page next to the PNG")
    parser.add_argument("--title", default="qr code", help="viewer page title")
    args = parser.parse_args()

    if args.file:
        with open(args.file, "r", encoding="utf-8") as handle:
            data = handle.read().strip()
    elif args.text:
        data = args.text
    else:
        parser.error("provide text or --file")
    if not data:
        parser.error("empty input")

    out = os.path.abspath(args.out)
    parent = os.path.dirname(out)
    if parent:
        os.makedirs(parent, exist_ok=True)
    segno.make(data, error="m").save(out, scale=args.scale, border=args.border)
    print(out)

    if args.html:
        page = os.path.splitext(out)[0] + ".html"
        with open(page, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(VIEWER.format(title=html.escape(args.title), name=html.escape(os.path.basename(out))))
        print(page)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
