# qr-code-skill

Offline QR code generator shipped as an agent skill: a command-line generator that needs no network, plus a `SKILL.md` an agent host can load. This folder is the source of truth; the copy used here lives under `~/.config/opencode/skills\qr`.

Source repo: https://github.com/chezhenkie/qr-code-skill, private.

## Layout

| Path | What it is |
|---|---|
| `SKILL.md` | Skill contract: frontmatter (`name: qr`, description, version) and the instructions an agent reads. |
| `python\make-qr.py` | The generator, shipped today. Text or file in, PNG out, optional viewer page. |
| `tools\gate.py` | 35-check acceptance gate. Rebuilds `tools\.artifacts` on every run and exits non-zero on any failure. |
| `tools\argv_matrix.py` | CLI parity harness: 27 cases frozen in `spec\argv-matrix.json`, normalised for program name and scratch paths. |
| `spec\argv-matrix.json` | Frozen CLI surface from the Python line: exit code, stdout and stderr for each case. |
| `.github\workflows\ci.yml` | Runs the gate and the CLI matrix on Windows and Linux, builds both binaries, and proves the Linux one runs in Alpine. |
| `rust-src\` | The Rust crate from P1 of the port. Empty until then. |
| `spec\qr-spec.md` | Behaviour contract: requirements QR-01 to QR-22, non-functional requirements, interfaces, distribution, states, validation map, gates and open questions. |
| `docs\` | Process manuals: spec sheet design and review, audit process, changelog and release, Python to Rust pitfalls. |
| `seed\` | The original Desktop seed (`make-qr.py`, `README.md`, `skill- doc-needs-editing.md`), moved here unedited. Historical only, never a build input. |
| `CHANGELOG.md` | Version chapters, newest first. |
| `LICENSE` | MIT. |
| `bin\` | Per-platform builds from P3: `qr.exe` and `qr`. Committed, because an install is a folder copy. |

The installed skill is `SKILL.md` plus the generator. Tools, spec, docs, seed and source folders are never shipped.

The version has one home: `metadata.version` in `SKILL.md` until the port, then the crate manifest alone. The changelog chapter and any later tag derive from it. Current 0.7.1; the Rust line releases as 0.8.0.

## Install

The skill folder holds `SKILL.md` and the generator. Copy both into the skills directory of the host. The path below is OpenCode's on Windows; a host that loads a `SKILL.md` folder needs the same two files, and the generator also runs on its own from a shell.

```powershell
Copy-Item .\SKILL.md, .\python\make-qr.py "$env:USERPROFILE\.config\opencode\skills\qr\" -Force
```

Reload the host afterwards. Keep the folder name `qr`: hosts that take the skill id from the folder name, OpenCode among them, read it there, and `SKILL.md` repeats it in the frontmatter.

Without any host: `python python\make-qr.py "https://example.com" -o .\out.png`.

## Dependency

Generator: `python -m pip install --user segno` (tested with segno 1.6.6 on Python 3.14.6). Pure Python, no network at runtime.

Gate only: `Pillow` and `zxing-cpp`, used to decode the produced PNG and compare the text. Neither is needed by the generator.

## Gate

```powershell
python tools\gate.py
```

Covers 35 checks. Input and output: text and file input, exact decode of both, whitespace stripping including the CPython strip set, CRLF fidelity, the two printed paths, overwrite, nested parent creation, and the absolute path. Symbol: level M and a regular symbol for short and long payloads, pixel size at scale 10 and 12, the IEND check on a truncated file. Page: references the PNG, hides the payload, carries the viewport tag, escapes the title and never encodes it. Errors: text plus file, an empty text argument with a file, no source, empty text, oversize input (2331-byte limit at level M), scale 0, border 17, a missing or non-UTF-8 file, and a blocked output or page path.

CI in `.github\workflows\ci.yml` runs this gate and the CLI matrix on Windows and Linux, builds the crate for both targets, and checks the Linux binary with `readelf` and inside an `alpine:3.21` container. Both binaries are built on the runners, so no local Rust toolchain or WSL is needed.

## Behaviour

Error correction is fixed at M and every symbol is a regular QR code, never Micro QR. `--scale` 1-100 defaults to 10 pixels per module; `--border` 0-16 defaults to 4 quiet-zone modules. `-f` reads UTF-8 without translating line endings, then strips surrounding whitespace. After writing, the script checks the PNG header against the symbol size and requires the IEND chunk, and removes the file and exits 1 on either failure, so a printed path always points at a complete image of the intended module count. Any filesystem failure ends with one `FAIL read:` or `FAIL write:` line and exit 1, never a traceback. Output paths resolve from the current directory and are printed absolute.

The viewer page loads the PNG by file name, scales it to 70vmin (the shorter side of the viewport), and never contains the encoded text.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Create it only when asked, keep it inside the project, delete it after scanning, and never send it to an online decoder.

## Port

The Rust line replaces the Python generator at 0.8.0: one console binary for Windows and one statically linked musl binary for Linux, both inside the `qr` skill folder, with no interpreter on the target. The crate lives in `rust-src\` and the built files in `bin\`. Python is deleted at the port and kept only as history. Behaviour requirements are in `spec\qr-spec.md`; the traps to avoid are in `docs\python-to-rust-refactor-pitfalls.md`. The P1 gate compares the port against `spec\argv-matrix.json` with `tools\argv_matrix.py`, and against segno's symbol choice over `spec\corpus.txt` with `tools\parity.py`. One known difference: text that Shift-JIS could encode is written as UTF-8 bytes, so the symbol can be denser; kanji mode is not reproduced.

## Status

2026-11-13: built and gated here, installed into the local skills folder, and confirmed as a loadable skill. Seed folder on the Desktop removed after the move.

2026-11-15: put under git and pushed to a private repo. Installed files still hash-equal to the source. Gate passes 15 of 15.

2026-11-15: process docs, spec sheet, changelog and license added. Layout split into `python\`, `tools\` and `rust-src\`; loose scratch output removed. Version set to 0.7.0, with 0.8.0 reserved for the Rust line.

2026-11-15: 0.7.1 closes the audit findings; the gate passes 35 of 35 and the installed copy is hash-equal to the source.
