# Requirements

What a host needs to run the generator, and what a development machine needs to check or rebuild it. The behaviour contract is `spec/qr-spec.md`.

## Supported hosts

| Host | Binary |
|---|---|
| Windows x86_64 | `bin/qr.exe` |
| Windows ARM64 | `bin/qr-arm64.exe` |
| Linux x86_64 | `bin/qr` |
| Linux ARM64 | `bin/qr-arm64` |
| macOS ARM64 | `bin/qr-macos-arm64` |

Intel macOS is not shipped; build from `rust-src/`.

## Runtime

- No network access.
- No interpreter, package manager, runtime library or configuration file.
- No temporary directory use. A run with no output option writes nothing; save mode writes only the named PNG and the optional page.
- A writable output path in save mode; missing parent folders are created.
- File input through `-f` is limited to 1 MiB of UTF-8.

## Terminal display

- A monospaced terminal wider than the symbol, quiet zone included. A wrapped line cannot be scanned.
- A screen bright enough for the camera. The renderer paints black modules on a white background, so the usual dark-on-light polarity holds.

## Saving and scanning

- `--scale` from 1 to 100 pixels per module; 10 is the default and 16 is a good choice for a phone camera.
- A quiet zone of 4 or more modules. `--border` accepts 0 to 16.
- A browser that loads the PNG from the same folder when the viewer page is used.

## Secrets

A QR of a token or password is a photographable secret. Prefer the terminal mode, keep saved files inside the project, and delete them after the scan.

## Verification and build

- Python 3.14 for `tools/`, with Pillow, zxing-cpp and segno.
- Rust only in GitHub Actions, on the runners: windows-latest, windows-11-arm, ubuntu-latest, ubuntu-24.04-arm and macos-latest.
- Docker only in CI, for the Alpine checks. No local Docker, WSL or Rust toolchain is needed.
- The gate and the matrix run against the shipped binary, so using the skill needs no compiler.
