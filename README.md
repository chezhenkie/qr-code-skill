# QR code

An agent skill that turns short text or a UTF-8 file into an offline QR code PNG, with an optional viewer page sized for scanning from a phone screen. The generator is a native binary: a Windows console executable and a statically linked Linux x86_64 binary ship in the repository, so the host needs no interpreter, package manager or network access. Error correction is fixed at level M, every symbol is a regular QR code, and the viewer page never contains the encoded text.

## Samples

Click a sample to open the full-size image.

| QR code | Viewer page |
|:---:|---|
| [<img src="docs/demo-qr.png" width="220" alt="QR code for https://example.com">](docs/demo-qr.png) | [<img src="docs/demo-page.png" width="220" alt="Viewer page with the QR code and its caption">](docs/demo-page.png) |
| Text input, 25 modules, scale 10 | `--html` page, image at 70vmin |

Both samples come from `bin/qr.exe "https://example.com" -o docs/demo-qr.png --scale 10 --html --title "Scan with the phone camera"`. The caption is a page title and never part of the code.

## Files

| File | Purpose |
|---|---|
| `SKILL.md` | Agent entry point: contract, flags, placement and decode recipe |
| `bin/qr.exe` | Windows console binary |
| `bin/qr` | Linux x86_64 binary, statically linked |
| `rust-src/` | The crate: command line, encoder, PNG writer, viewer template |
| `tools/gate.py` | 36-check acceptance gate over the shipped binary |
| `tools/argv_matrix.py` | 27-case command-line harness against the frozen fixture |
| `tools/parity.py` | Symbol parity against segno over `spec/corpus.txt` |
| `spec/` | Behaviour contract, frozen command-line fixture, parity corpus |
| `docs/` | Process manuals and the sample images |
| `seed/` | The original Python script, kept as history |
| `CHANGELOG.md` | Version history |

## Install

The skill payload is `SKILL.md` and `bin/`. Clone the repository, then copy both into the skills folder for your agent and name the folder `qr`. The folder name must match the `name` in `SKILL.md`.

```sh
git clone https://github.com/chezhenkie/qr-code-skill.git
```

| Agent | All projects | One project |
|---|---|---|
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` |
| Codex | `~/.agents/skills` | `.agents/skills` |
| Pi | `~/.pi/agent/skills` | `.pi/skills` |
| OMP | `~/.omp/agent/skills` | `.omp/skills` |

```powershell
Copy-Item -Recurse .\SKILL.md, .\bin "$HOME\.config\opencode\skills\qr"
```

```sh
mkdir -p ~/.config/opencode/skills/qr
cp -r SKILL.md bin ~/.config/opencode/skills/qr
```

`~` is your home directory; if the shell does not expand it, use the full path. The [v0.8.0 release](https://github.com/chezhenkie/qr-code-skill/releases/tag/v0.8.0) carries the same files as one archive per platform. Restart the agent if the skill does not appear.

## Use

Ask for the code and name the output file. The agent picks the binary for the platform, runs it and reports the printed path.

```text
Make a QR code for https://example.com in the project folder.
Encode notes/token.txt as a QR code with a viewer page for the phone, scale 12.
```

| Agent | Invocation |
|---|---|
| OpenCode | `@qr` |
| Codex | `$qr` |
| Pi | `/skill:qr` |
| OMP | `/skill:qr` |

Direct use, from the repository root:

```powershell
.\bin\qr.exe "https://example.com" -o .\out.png
.\bin\qr.exe -f .\notes.txt -o .\notes.png --scale 12 --html --title "notes"
```

```sh
./bin/qr "https://example.com" -o ./out.png
```

Each run prints the absolute path of the PNG on stdout and, with `--html`, the page path after it. Arguments are checked before a file is written. A filesystem failure prints one `FAIL read:` or `FAIL write:` line and exits 1; text beyond the capacity at level M prints one `FAIL store:` line and leaves no file; a usage error prints the help text and exits 2.

## Options

| Option | Meaning | Default |
|---|---|---|
| `text` | Text to encode; leave it out when using `-f` | |
| `-f, --file FILE` | Read the text from a UTF-8 file, line endings kept | |
| `-o, --out OUT` | PNG path | `qr.png` |
| `--scale SCALE` | Pixels per module, 1 to 100 | 10 |
| `--border BORDER` | Quiet zone in modules, 0 to 16 | 4 |
| `--html` | Write a viewer page beside the PNG, named after it | off |
| `--title TITLE` | Caption on the viewer page | none |
| `-h, --help` | Print the usage text | |
| `--version` | Print the version; the flag is not listed in the help text | |

Text and file input are stripped of surrounding whitespace before encoding. The PNG is written by the tool itself as a 1-bit greyscale image, then the header is checked against the symbol size and the closing chunk is required; a mismatch removes the file and exits 1, so a printed path points at a complete image of the intended module count. The page loads that image by file name, scales it to 70vmin of the shorter viewport side, and carries a viewport tag so the size holds on a phone.

## Cleanup

The generator writes only the files you name: the PNG at `-o` and, with `--html`, the page beside it. It uses no temporary folder, cache or log, so no QR image is left anywhere else on disk.

```powershell
Remove-Item .\qr\out.png, .\qr\out.html
```

```sh
rm qr/out.png qr/out.html
```

Deletion is a manual step; nothing removes the files later. A payload over capacity writes nothing, a failed write or PNG check removes the PNG, and a failed page write leaves the PNG so the run can be retried. Delete a secret as soon as the scan is confirmed, without copying it elsewhere first.

## Check

Run from the repository root:

```sh
python tools/gate.py
python tools/argv_matrix.py
```

The gate runs 36 checks over the shipped binary: decode at level M, dimensions at two scales, the page contents, the strip boundaries, error paths, and a truncated PNG. The matrix compares 27 command-line cases with the frozen fixture. Both need Python 3; the gate also needs Pillow and zxing-cpp and exits 2 when they are missing. Both tools write their fixtures under `tools/.artifacts` and delete them on exit.

CI runs the gate and the matrix on Windows and Linux, builds the crate for both targets, checks that the Linux binary has no dynamic dependency, and runs it on `alpine:3.21`. No local Rust toolchain is needed.

## Limits

- Level M only. Byte mode holds 2331 bytes of text, digits and uppercase fit more; longer input fails with one line.
- `--scale` accepts 1 to 100 and `--border` 0 to 16. A quiet zone of 4 or more modules scans best.
- Long text makes a dense symbol, so raise `--scale` for screen scanning. 2331 bytes at scale 100 is 18500 pixels per side.
- Text is stored as UTF-8. Text that a Shift-JIS encoder would place in kanji mode is written as bytes, so the symbol can need more versions.
- The viewer page loads the PNG from the same folder; copy both files together.
- Prebuilt binaries cover Windows x86_64 and Linux x86_64. `cargo` builds other targets from `rust-src/`.

## Secrets

A QR code that encodes a token or a password is a photographable secret. Generate it on request, keep it inside the project, delete it after scanning, and never send it to an online decoder. Scanner apps keep history, and cloud sync may upload the image.

## License

MIT
