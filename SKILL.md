---
name: qr
description: "Generate an offline QR code PNG from text or a file, with an optional local viewer page sized for scanning from a screen. Use when a URL, token, Wi-Fi string, contact card, or other short text must be shown as a QR code or saved as an image, and when generation must stay offline. Not for reading or decoding QR codes."
license: MIT
metadata:
  version: "0.7.0"
---

# QR

Turn short text into a scannable QR code PNG without network access, optionally with a viewer page sized for a phone camera.

## Dependency

One pure-Python package, installed once at user level:

```powershell
python -c "import segno"
python -m pip install --user segno
```

The first command must exit 0; install when it raises `ModuleNotFoundError`.

Tested with segno 1.6.6 on Python 3.14.6. The script fails with a clear `FAIL store:` line on stderr and exit 1 when the text exceeds the symbol capacity at error level M (byte mode tops out at 2331 bytes; digits and uppercase text fit more).

## Usage

In the installed skill, run from the folder that holds this file, or call the script through its absolute path. Output paths resolve from the current directory.

```powershell
python make-qr.py "https://example.com" -o .\qr\out.png
python make-qr.py -f .\secret.txt -o .\qr\secret.png --scale 12 --html --title "phone token"
```

- `-f/--file` reads the text and strips surrounding whitespace; give either text or a file, not both.
- `-o/--out` is the PNG path, default `qr.png`; the script prints the absolute path on stdout.
- `--scale` is pixels per module, 1 to 100, default 10.
- `--border` is the quiet zone in modules, 0 to 16, default 4.
- `--html` writes a viewer page next to the PNG, named after it, and prints its absolute path on stdout after the PNG path.
- `--title` sets the viewer caption only; it is never encoded.

Error correction is fixed at level M. The viewer page loads the image by file name, scales it to 70vmin (the shorter side of the viewport), and never contains the encoded text.

## Contract

The script checks the written PNG against the symbol size before it prints anything and removes the file and exits 1 on a mismatch, so a printed path always matches the intended module count. Arguments are validated before any file is written.

## Placement

Write the PNG into the project being worked on, for example `qr\` in the project root. Do not write to a system temp directory or the Desktop.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Create it only when asked, keep it inside the project, and delete it after it is scanned; scanner apps keep history and cloud sync may upload it. Never send such an image to an online decoder.

## Limits

Keep the quiet zone at 4 or more modules; a tight crop fails on many scanners. Long text produces a dense symbol (version 40 is 177 modules), so raise `--scale` to 16 for screen scanning. A dark-on-light code on a matte screen scans best; invert nothing.

## Verification

A structural check runs inside the script. For a full decode gate, with the optional `zxing-cpp` package installed:

```powershell
python -c "import sys, zxingcpp; from PIL import Image; r = zxingcpp.read_barcode(Image.open(sys.argv[1])); print(r.text if r else 'NO-DECODE')" .\qr\out.png
```

Use it only on non-secret test content.
