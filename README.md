# qr-code-skill

Offline QR code generator, packaged as an OpenCode skill. This folder is the source of truth; the live install is a copy under `~/.config/opencode/skills\qr`.

Source repo: https://github.com/chezhenkie/qr-code-skill, private.

## Layout

| Path | What it is |
|---|---|
| `SKILL.md` | Skill contract: frontmatter (`name: qr`, description, version) and the instructions an agent reads. |
| `python\make-qr.py` | The generator, shipped today. Text or file in, PNG out, optional viewer page. |
| `tools\gate.py` | 15-check acceptance gate. Rebuilds `tools\.artifacts` on every run and exits non-zero on any failure. |
| `rust-src\` | The Rust crate from P1 of the port. Empty until then. |
| `spec\qr-spec.md` | Behaviour contract: requirements QR-01 to QR-22, non-functional requirements, interfaces, distribution, states, validation map, gates and open questions. |
| `docs\` | Process manuals: spec sheet design and review, audit process, changelog and release, Python to Rust pitfalls. |
| `seed\` | The original Desktop seed (`make-qr.py`, `README.md`, `skill- doc-needs-editing.md`), moved here unedited. Historical only, never a build input. |
| `CHANGELOG.md` | Version chapters, newest first. |
| `LICENSE` | MIT. |
| `bin\` | Per-platform builds from P3: `qr.exe` and `qr`. Committed, because an install is a folder copy. |

The installed skill is `SKILL.md` plus the generator. Tools, spec, docs, seed and source folders are never shipped.

The version has one home: `metadata.version` in `SKILL.md` until the port, then the crate manifest alone. The changelog chapter and any later tag derive from it. Current 0.7.0; the Rust line releases as 0.8.0.

## Install

```powershell
Copy-Item .\SKILL.md, .\python\make-qr.py "$env:USERPROFILE\.config\opencode\skills\qr\" -Force
```

Reload OpenCode afterwards. The directory name must stay `qr`: the skill id is the folder name, and `SKILL.md` repeats it in the frontmatter.

## Dependency

Generator: `python -m pip install --user segno` (tested with segno 1.6.6 on Python 3.14.6). Pure Python, no network at runtime.

Gate only: `Pillow` and `zxing-cpp`, used to decode the produced PNG and compare the text. Neither is needed by the generator.

## Gate

```powershell
python tools\gate.py
```

Covers: text and file input, exact decode of both, whitespace stripping, the two printed paths, viewer page references the PNG and never contains the encoded text, pixel size at scale 10 and 12, and rejection of text plus file, empty text, oversize input (2331-byte limit at error level M), scale 0, and border 17.

## Behaviour

Error correction is fixed at M. `--scale` 1-100 defaults to 10 pixels per module; `--border` 0-16 defaults to 4 quiet-zone modules. After writing, the script checks the PNG header against the symbol size, removes the file and exits 1 on a mismatch, so a printed path always matches the intended module count. Output paths resolve from the current directory and are printed absolute.

The viewer page loads the PNG by file name, scales it to 70vmin (the shorter side of the viewport), and never contains the encoded text.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Create it only when asked, keep it inside the project, delete it after scanning, and never send it to an online decoder.

## Port

The Rust line replaces the Python generator at 0.8.0: one console binary for Windows and one statically linked musl binary for Linux, both inside the `qr` skill folder, with no interpreter on the target. The crate lives in `rust-src\` and the built files in `bin\`. Python is deleted at the port and kept only as history. Behaviour requirements are in `spec\qr-spec.md`; the traps to avoid are in `docs\python-to-rust-refactor-pitfalls.md`.

## Status

2026-11-13: built and gated here, installed at `~/.config/opencode/skills\qr`, and confirmed as a loadable skill. Seed folder on the Desktop removed after the move. No open defects.

2026-11-15: put under git and pushed to a private repo. Installed files still hash-equal to the source. Gate passes 15 of 15.

2026-11-15: process docs, spec sheet, changelog and license added. Layout split into `python\`, `tools\` and `rust-src\`; loose scratch output removed. Version set to 0.7.0, with 0.8.0 reserved for the Rust line.
