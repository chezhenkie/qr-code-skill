# Changelog

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
