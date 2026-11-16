# Changelog

## 1.1.0

### New features

#### macOS ARM64

* Native binary, `bin/qr-macos-arm64`. Intel Macs build from source.

### Changes

* `SKILL.md` names the binary for the host architecture, macOS included.

## 1.0.0

### New features

#### All platforms

* Native ARM64 binaries: `bin/qr-arm64.exe` for Windows ARM64 and `bin/qr-arm64` for Linux ARM64, statically linked like the x86_64 binary.

### Changes

* `SKILL.md` names the binary for the host architecture.
* Version 1.0.0 marks the stable install model and command line.

## 0.10.1

### Changes

* The help text and the usage line list `--version`.

## 0.10.0

### New features

#### All platforms

* `--save` writes the PNG at the default path, for a run that wants a file without naming one.
* `-o`, `--scale`, `--html` and `--title` select save mode, so a script that names a file keeps working.

### Changes

* With no output option the QR code is printed to the terminal and nothing is written to disk.
* `--term` is removed; the terminal display is the default.

## 0.9.0

### New features

#### All platforms

* `--term` prints the QR code to the terminal and writes no file, so a code that is scanned once leaves nothing to delete.
* The terminal rendering keeps the quiet zone and paints a light background behind black modules, so it scans from a dark terminal.
* `--term` refuses `--out`, `--html`, `--title` and `--scale` with exit 2.

### Changes

* The help text and the usage line list `--term`.

## 0.8.0

### New features

#### All platforms

* The generator is a native binary, `bin\qr.exe` on Windows and a statically linked `bin/qr` for Linux x86_64, so the target needs no interpreter or package at runtime.
* `--version` prints the version the binary was built from.
* One source tree and one manifest build both binaries.

### Changes

* The Python generator is removed; the skill ships `SKILL.md` and `bin/`.
* The PNG is written by the tool itself as a 1-bit greyscale image. Symbols, dimensions and viewer pages match 0.7.1; the PNG bytes differ because the encoder changed.
* `-o` writes a PNG whatever the extension; the 0.7.1 rule that required `.png` is gone.

## 0.7.1

### Fixes

#### All platforms

* Error correction stays at level M: the encoder no longer raises it when the symbol has spare capacity.
* Symbols are regular QR codes; payloads of two to four characters no longer produce a Micro QR symbol.
* `-f` keeps line endings as stored, so a file holding CRLF encodes CRLF.
* A missing or non-UTF-8 input file fails with one `FAIL read:` line and exit 1 instead of a traceback.
* An unwritable output or page path fails with one `FAIL write:` line and exit 1, and the PNG path is printed only after every file is written.
* The PNG must end with an IEND chunk before a path is printed.
* An empty text argument next to `-f` is rejected as a conflicting source.
* The viewer page carries a viewport tag, so 70vmin resolves against the phone screen.

## 0.7.0

### New features

#### All platforms

* Offline QR code generator: text or file in, PNG out, no network.
* Output path, module scale and quiet-zone options; error correction fixed at level M.
* Optional viewer page sized for scanning from a screen, carrying no script and no payload text.
* Post-write header check: a size mismatch removes the file and fails the run.
