# qr-code-skill

Offline QR code generator, packaged as an OpenCode skill. This folder is the source of truth; the live install is a copy under `~/.config/opencode/skills\qr`.

Source repo: https://github.com/chezhenkie/qr-code-skill, private.

## Files

| Path | What it is |
|---|---|
| `SKILL.md` | Skill contract: frontmatter (`name: qr`, description) and the instructions an agent reads. |
| `make-qr.py` | The generator. Text or file in, PNG out, optional viewer page. |
| `spec\qr-spec.md` | Behaviour contract: requirements QR-01 to QR-21, non-functional requirements, interfaces, states, validation map, gates and open questions. |
| `docs\` | Process manuals: spec sheet design and review, audit process, changelog and release, Python to Rust pitfalls. |
| `gate\gate.py` | 15-check acceptance gate. Rewrites `gate\out` on every run and exits non-zero on any failure. |
| `gate\out` | Scratch output of the last gate run. Disposable. |
| `seed\` | The original Desktop seed (`make-qr.py`, `README.md`, `skill- doc-needs-editing.md`), moved here unedited. Historical only; the root copy is current. |
| `CHANGELOG.md` | Version chapters, newest first. |
| `LICENSE` | MIT. |

Only `SKILL.md` and `make-qr.py` belong in the installed skill directory. The gate and the seed stay out of it so the skill stays two files.

The version has one home: `metadata.version` in `SKILL.md`. The changelog chapter and any later tag derive from it.

## Install

```powershell
Copy-Item .\SKILL.md, .\make-qr.py "$env:USERPROFILE\.config\opencode\skills\qr\" -Force
```

Reload OpenCode afterwards. The directory name must stay `qr`: the skill id is the folder name, and `SKILL.md` repeats it in the frontmatter.

## Dependency

Generator: `python -m pip install --user segno` (tested with segno 1.6.6 on Python 3.14.6). Pure Python, no network at runtime.

Gate only: `Pillow` and `zxing-cpp`, used to decode the produced PNG and compare the text. Neither is needed by the generator.

## Gate

```powershell
python gate\gate.py
```

Covers: text and file input, exact decode of both, whitespace stripping, the two printed paths, viewer page references the PNG and never contains the encoded text, pixel size at scale 10 and 12, and rejection of text plus file, empty text, oversize input (2331-byte limit at error level M), scale 0, and border 17.

## Behaviour

Error correction is fixed at M. `--scale` 1-100 defaults to 10 pixels per module; `--border` 0-16 defaults to 4 quiet-zone modules. After writing, the script checks the PNG header against the symbol size, removes the file and exits 1 on a mismatch, so a printed path is always a complete image. Output paths resolve from the current directory and are printed absolute.

The viewer page loads the PNG by file name, scales it to 70% of the viewport, and never contains the encoded text.

## Secrets

A QR that encodes a token, password, or provisioning string is a photographable secret. Create it only when asked, keep it inside the project, delete it after scanning, and never send it to an online decoder.

## Status

2026-11-13: built and gated here, installed at `~/.config/opencode/skills\qr`, and confirmed as a loadable skill. Seed folder on the Desktop removed after the move. No open defects.

2026-11-15: put under git and pushed to a private repo. Installed files still hash-equal to the source. Gate passes 15 of 15.

2026-11-15: process docs added under `docs\`; spec sheet, changelog and license added. Version moved to `1.0.0` in `SKILL.md`.
