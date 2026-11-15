---
name: qr
description: "Generate an offline QR code from text or a file: print it to the terminal by default with nothing written to disk, or save a PNG with --save, optionally with a local viewer page sized for scanning from a screen. Use when a URL, token, Wi-Fi string, contact card, or other short text must be shown as a QR code or saved as an image, and when generation must stay offline. Not for reading or decoding QR codes."
license: MIT
---

# QR

Turn short text into a scannable QR code without network access. The default is a screen display: the symbol is painted to the terminal and no file is written. `--save` writes a PNG, optionally with a viewer page sized for a phone camera.

## Install

The skill folder holds this file and `bin/`. The generator is a native binary, so the target needs no interpreter, package manager or build step.

- Windows: `bin\qr.exe`
- Linux x86_64: `bin/qr`, statically linked, so it runs on Alpine and other images with no runtime package.

Both binaries come from one source tree and report the same version with `--version`.

## Usage

Run the binary from the folder that holds this file, or call it by absolute path. Output paths resolve from the current directory.

```powershell
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" "https://example.com"
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" --save -f .\secret.txt -o .\qr\secret.png --scale 12 --html --title "phone token"
```

On Linux the binary is `~/.config/opencode/skills/qr/bin/qr` with the same arguments.

- With no output option the symbol goes to stdout and nothing is written.
- `--save` writes the PNG, at `-o` when given, default `qr.png`; the absolute path is printed on stdout.
- `-o/--out`, `--scale`, `--html` and `--title` imply `--save`.
- `-f/--file` reads the file as UTF-8 and keeps line endings as stored, then strips surrounding whitespace; give either text or a file, not both.
- `--scale` is pixels per module, 1 to 100, default 10; it applies to the PNG only.
- `--border` is the quiet zone in modules, 0 to 16, default 4; it applies to both modes.
- `--html` writes a viewer page next to the PNG, named after it with the extension replaced by `.html`, and prints its absolute path on stdout after the PNG path.
- `--title` sets the viewer caption only; it is never encoded.
- `--version` prints the version and exits 0.

Error correction is fixed at level M, and every symbol is a regular QR code, never Micro QR. The viewer page loads the image by file name, scales it to 70vmin (the shorter side of the viewport), and never contains the encoded text.

## Show on screen

This is the default and the main use. The symbol is rendered from memory, written to stdout and nothing on disk is touched: no PNG, no page, no directory, no temporary copy, no log. A code that is scanned once needs no cleanup.

```powershell
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" "https://example.com"
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" --border 2 "https://example.com"
```

The output is UTF-8 text with a colour prefix: black modules on a painted bright-white background, two module rows per text row, quiet zone included, so a dark terminal still shows the polarity a scanner expects.

It needs a monospaced terminal, a window wide enough that no line wraps, and enough screen brightness for the camera. A wrapped or recoloured rendering does not scan; widen the window instead of scaling it.

## Save a PNG

`--save` writes the image, and so does any option that only makes sense for a file: `-o`, `--scale`, `--html`, `--title`.

```powershell
& "$env:USERPROFILE\.config\opencode\skills\qr\bin\qr.exe" --save "https://example.com" -o .\qr\out.png
```

Write the PNG into the project being worked on, for example `qr\` in the project root. Do not write to a system temp directory or the Desktop.

## Contract

In save mode the binary checks the written PNG header against the symbol size and requires the closing IEND chunk before it prints anything, so a printed path always points at a complete image of the intended module count. Arguments are validated before any file is written, and a filesystem failure ends with one `FAIL read:` or `FAIL write:` line on stderr and exit 1, never a traceback.

Text over the capacity at level M fails with one `FAIL store:` line and leaves no file; byte mode tops out at 2331 bytes, digits and uppercase text fit more. The terminal display reports a failed stdout write as one `FAIL write: stdout` line.

## Exit and cleanup

- Exit 0: terminal mode has painted the symbol; save mode has printed the absolute paths and the PNG passed the size and IEND check.
- Exit 1: one `FAIL` line on stderr. A payload over capacity writes nothing, a failed write or PNG check removes the PNG, and a failed page write leaves the PNG for a retry.
- Exit 2: usage error, nothing written.

Nothing removes saved files automatically. After the scan, delete the PNG and the page: `Remove-Item .\qr\out.png, .\qr\out.html` on Windows, `rm qr/out.png qr/out.html` on Linux. Delete a secret as soon as the scan is confirmed, without copying the files elsewhere first. A process stopped mid-write can leave a partial PNG at the output path; delete that path as well. The default terminal mode leaves nothing to delete.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Prefer the terminal mode, which leaves no artefact on disk. In save mode, create the files only when asked, keep them inside the project, and delete them after the scan; scanner apps keep history and cloud sync may upload the image. Never send such an image to an online decoder.

## Limits

Keep the quiet zone at 4 or more modules; a tight crop fails on many scanners. Long text produces a dense symbol (version 40 is 177 modules), so raise `--scale` to 16 for screen scanning. The range itself allows far more than a scanner needs: 2331 bytes at `--scale 100` is 18500 px per side, about 342 megapixels. A dark-on-light code on a matte screen scans best; invert nothing.

Terminal rendering is one column per module plus the quiet zone, so the window must be wider than the code; a wrapped line cannot be scanned.

## Verification

The binary performs a structural check itself in save mode. For a full decode check on a development machine, with the optional `zxing-cpp` package installed:

```powershell
python -c "import sys, zxingcpp; from PIL import Image; r = zxingcpp.read_barcode(Image.open(sys.argv[1])); print(r.text if r else 'NO-DECODE')" .\qr\out.png
```

Use it only on non-secret test content.
