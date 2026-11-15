---
name: qr
description: "Generate an offline QR code PNG from text or a file, with an optional local viewer page sized for scanning from a screen. Use when a URL, token, Wi-Fi string, contact card, or other short text must be shown as a QR code or saved as an image, and when generation must stay offline. Not for reading or decoding QR codes."
license: MIT
---

# QR

Turn short text into a scannable QR code PNG without network access, optionally with a viewer page sized for a phone camera.

## Install

The skill folder holds this file and `bin/`. The generator is a native binary, so the target needs no interpreter, package manager or build step.

- Windows: `bin\qr.exe`
- Linux x86_64: `bin/qr`, statically linked, so it runs on Alpine and other images with no runtime package.

Both binaries come from one source tree and report the same version with `--version`.

## Usage

Run the binary from the folder that holds this file, or call it by absolute path. Output paths resolve from the current directory.

```powershell
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" "https://example.com" -o .\qr\out.png
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" -f .\secret.txt -o .\qr\secret.png --scale 12 --html --title "phone token"
```

On Linux the binary is `~/.config/opencode/skills/qr/bin/qr` with the same arguments.

- `-f/--file` reads the file as UTF-8 and keeps line endings as stored, then strips surrounding whitespace; give either text or a file, not both.
- `-o/--out` is the PNG path, default `qr.png`; the image is always written as PNG, and the absolute path is printed on stdout. Keeping the `.png` extension keeps the file name honest and the viewer page name obvious.
- `--scale` is pixels per module, 1 to 100, default 10.
- `--border` is the quiet zone in modules, 0 to 16, default 4.
- `--html` writes a viewer page next to the PNG, named after it with the extension replaced by `.html`, and prints its absolute path on stdout after the PNG path.
- `--title` sets the viewer caption only; it is never encoded.
- `--version` prints the version and exits 0.

Error correction is fixed at level M, and every symbol is a regular QR code, never Micro QR. The viewer page loads the image by file name, scales it to 70vmin (the shorter side of the viewport), and never contains the encoded text.

## Contract

The binary checks the written PNG header against the symbol size and requires the closing IEND chunk before it prints anything, so a printed path always points at a complete image of the intended module count. Arguments are validated before any file is written, and a filesystem failure ends with one `FAIL read:` or `FAIL write:` line on stderr and exit 1, never a traceback. Text over the capacity at level M fails with one `FAIL store:` line and leaves no file; byte mode tops out at 2331 bytes, digits and uppercase text fit more.

## Placement

Write the PNG into the project being worked on, for example `qr\` in the project root. Do not write to a system temp directory or the Desktop.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Create it only when asked, keep it inside the project, and delete it after it is scanned; scanner apps keep history and cloud sync may upload it. Never send such an image to an online decoder.

## Limits

Keep the quiet zone at 4 or more modules; a tight crop fails on many scanners. Long text produces a dense symbol (version 40 is 177 modules), so raise `--scale` to 16 for screen scanning. The range itself allows far more than a scanner needs: 2331 bytes at `--scale 100` is 18500 px per side, about 342 megapixels. A dark-on-light code on a matte screen scans best; invert nothing.

## Verification

The binary performs a structural check itself. For a full decode check on a development machine, with the optional `zxing-cpp` package installed:

```powershell
python -c "import sys, zxingcpp; from PIL import Image; r = zxingcpp.read_barcode(Image.open(sys.argv[1])); print(r.text if r else 'NO-DECODE')" .\qr\out.png
```

Use it only on non-secret test content.
