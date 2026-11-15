#!/usr/bin/env python3
"""Generate an offline QR code PNG, optionally with a viewer page.

No network access. Requires the segno package: python -m pip install --user segno
"""
import argparse
import html
import os
import struct
import sys

import segno
from segno.encoder import DataOverflowError

VIEWER = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
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

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def png_probe(path: str):
    """Return (IHDR size or None, whether the file ends with an IEND chunk)."""
    with open(path, "rb") as handle:
        head = handle.read(24)
        end = os.fstat(handle.fileno()).st_size
        tail = b""
        if end >= 12:
            handle.seek(end - 12)
            tail = handle.read(12)
    complete = len(tail) == 12 and tail[4:8] == b"IEND"
    if len(head) < 24 or head[:8] != PNG_SIGNATURE or head[12:16] != b"IHDR":
        return None, complete
    return struct.unpack(">II", head[16:24]), complete


def discard(path: str) -> None:
    try:
        os.remove(path)
    except OSError:
        pass


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate an offline QR code PNG.")
    parser.add_argument("text", nargs="?", help="text to encode; omit when using --file")
    parser.add_argument("-f", "--file", help="read the text from a file, stripped")
    parser.add_argument("-o", "--out", default="qr.png", help="output PNG path")
    parser.add_argument("--scale", type=int, default=10, help="pixels per module 1-100 (default 10)")
    parser.add_argument("--border", type=int, default=4, help="quiet zone in modules 0-16 (default 4)")
    parser.add_argument("--html", action="store_true", help="write a viewer page next to the PNG")
    parser.add_argument("--title", default="qr code", help="viewer page title")
    args = parser.parse_args()

    if not 1 <= args.scale <= 100:
        parser.error("--scale must be 1-100")
    if not 0 <= args.border <= 16:
        parser.error("--border must be 0-16")
    if args.file and args.text is not None:
        parser.error("give text or --file, not both")
    if args.file:
        with open(args.file, "r", encoding="utf-8") as handle:
            data = handle.read().strip()
    elif args.text is not None:
        data = args.text
    else:
        parser.error("provide text or --file")
    if not data:
        parser.error("empty input")

    out = os.path.abspath(args.out)
    parent = os.path.dirname(out)
    if parent:
        os.makedirs(parent, exist_ok=True)
    try:
        code = segno.make(data, error="m", boost_error=False, micro=False)
        code.save(out, scale=args.scale, border=args.border)
    except (DataOverflowError, ValueError) as exc:
        print(f"FAIL store: {exc}", file=sys.stderr)
        return 1

    want = code.symbol_size(scale=args.scale, border=args.border)
    got, complete = png_probe(out)
    if got != want:
        discard(out)
        print(f"FAIL png size {got} expected {want}", file=sys.stderr)
        return 1
    if not complete:
        discard(out)
        print("FAIL png tail: missing IEND", file=sys.stderr)
        return 1
    print(out)

    if args.html:
        page = os.path.splitext(out)[0] + ".html"
        with open(page, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(VIEWER.format(title=html.escape(args.title), name=html.escape(os.path.basename(out))))
        print(page)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
