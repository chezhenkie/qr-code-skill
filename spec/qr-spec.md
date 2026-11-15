# QR skill spec sheet

| Field | Value |
|---|---|
| Title | qr: offline QR code generator skill |
| Purpose | Pin observable behaviour of the skill, first for the Python line, then as the acceptance target for the Rust line that replaces it |
| Status | under review |
| Version | 0.7.1 Python, 0.8.0 Rust |
| Owner | skill maintainer |
| Stakeholders | agent operators who load the skill, reviewers of the port |
| Approvers | owner, reviewer |
| Related | SKILL.md, README.md, tools/gate.py, tools/argv_matrix.py, spec/argv-matrix.json, CHANGELOG.md, docs/specsheet-design-and-review-manual.md, docs/audit-process-manual.md, docs/changelog-build-and-release-manual.md, docs/python-to-rust-refactor-pitfalls.md |
| Scope | the shipped skill, SKILL.md and the generator; the gate is verification, not product |

## 1. Problem and goals

An agent that needs a scannable code for a URL, token, Wi-Fi string or contact card has no dependable way to produce one. Ad hoc attempts write throwaway scripts, call an online generator, or leave images in a temp folder that outlives the request. For a token or password payload a network round trip is a disclosure, and a code drawn without a quiet zone or at one pixel per module fails on many scanners.

Goals:

- G1: one command turns short text into a PNG that decodes back to the exact input.
- G2: an optional viewer page shows the code large enough to scan from a screen.
- G3: a failed run never prints a path to a broken or absent file.
- G4: install is a folder copy with no interpreter, package manager or build step on the target.

Non-goals: decoding, batch generation, SVG or EPS output, logos or branding, animated codes, a choice of error-correction level, network delivery, a GUI, ARM64 and macOS builds for 0.8.0. Decoding, other symbologies and image input are planned as separate crates after 0.8.0 and the public switch.

## 2. Functional requirements

QR-22 is a target for the 0.8.0 binary. The Python line has no `--version` flag, so the requirement is not claimed for it.

| Id | Required behaviour | Failure behaviour |
|---|---|---|
| QR-01 | A single positional argument is encoded unchanged: no strip, no case change, no re-encoding. | none |
| QR-02 | `-f PATH` reads the file as UTF-8, without line-ending translation, and strips leading and trailing whitespace before encoding. The whitespace set is CPython's, measured on 3.14.6: U+0009 to U+000D, U+001C to U+0020, U+0085, U+00A0, U+1680, U+2000 to U+200A, U+2028, U+2029, U+202F, U+205F, U+3000, 29 code points in 10 ranges. Rust `str::trim` follows Unicode White_Space and keeps U+001C to U+001F, so the port carries this table. | QR-19 |
| QR-03 | Text and `-f` are mutually exclusive. | exit 2, stderr `give text or --file, not both`, nothing written |
| QR-04 | Neither text nor `-f` is given. | exit 2, stderr `provide text or --file`, nothing written |
| QR-05 | The payload is empty, either as an empty argument or a file that strips to nothing. | exit 2, stderr `empty input`, nothing written |
| QR-06 | `-o PATH` sets the PNG path, default `qr.png`, resolved against the current directory; the absolute path is printed. | QR-19 |
| QR-07 | Missing parent directories of the output path are created, including nested ones. | QR-19 |
| QR-08 | `--scale` is an integer 1 to 100, default 10 pixels per module. | exit 2, stderr `--scale must be 1-100` |
| QR-09 | `--border` is an integer 0 to 16, default 4 quiet-zone modules. | exit 2, stderr `--border must be 0-16` |
| QR-10 | Error correction is fixed at level M and is not selectable. Symbols are regular QR, never Micro QR. | no option exists |
| QR-11 | A payload over the capacity of level M fails without leaving a file. | exit 1, stderr starts `FAIL store:` |
| QR-12 | The PNG is square, side `(modules + 2 * border) * scale` pixels, and decodes back to the payload. | QR-11, QR-13 |
| QR-13 | After writing, the PNG header dimensions are compared with the expected symbol size, and the file must end with an IEND chunk. | on a size mismatch the file is removed, exit 1, stderr `FAIL png size <got> expected <want>`; without IEND the file is removed, exit 1, stderr `FAIL png tail: missing IEND` |
| QR-14 | stdout carries only the absolute PNG path, then, with `--html`, the absolute page path on a second line. | no other stdout output on any path |
| QR-15 | `--html` writes a page named after the PNG stem, next to the PNG, referencing the image by file name; it contains no payload text, no script and no external asset; the image is scaled to 70vmin, the shorter side of the viewport. | QR-19 |
| QR-16 | `--title` sets the page title and caption only, HTML-escaped, and is never encoded. | none; any string is accepted |
| QR-17 | Exit 0 on success, 1 on a runtime failure, 2 on a usage error. | n/a |
| QR-18 | An existing output file is replaced. | QR-19 |
| QR-19 | Any filesystem failure, including an unreadable, missing or non-UTF-8 `-f` file, ends with a one-line error on stderr and exit 1, with no traceback and no partial file left behind. | n/a |
| QR-20 | `-h` and `--help` print usage on stdout and exit 0; an unknown option is a usage error. | exit 2 for unknown options |
| QR-21 | Standard input is never read. | n/a |
| QR-22 | `--version` prints the single version string and exits 0, on both platforms. | exit 0 always |

## 3. Non-functional requirements

Requirements marked 0.8.0 are targets for the Rust line and are not claimed for the Python line.

| Id | Requirement |
|---|---|
| NFR-01 | No network at runtime: no socket, DNS or HTTP. |
| NFR-02 | From 0.8.0 the generator needs no interpreter or package at runtime; until then it needs CPython 3.14 and segno. The gate's dependencies stay in the gate. |
| NFR-03 | Repository text is ASCII with LF endings; generated images and gate scratch are not committed. |
| NFR-04 | One PNG of a URL of 60 characters completes in under 1 s on the reference machine. Guess, to be measured at the port. |
| NFR-05 | Peak memory stays under 16 MiB for a 2000-byte payload. Guess, to be measured. |
| NFR-06 | Windows x86_64 and Linux x86_64 behave identically for file output, exit codes and file names. |
| NFR-07 | The same payload and options produce byte-identical PNG output on the same platform. |
| NFR-08 | The payload never reaches stdout, stderr, a log or a file name, and no telemetry is collected. |
| NFR-09 | From 0.8.0 the Linux binary is statically linked and carries no dynamic loader dependency; it starts on Alpine x86_64 with no package installed. |
| NFR-10 | From 0.8.0 the Windows binary starts with no runtime redistributable. Size under 4 MB per binary. Guess, to be measured. |
| NFR-11 | From 0.8.0 the build uses one source tree and one version for both binaries, with no per-platform source differences. |

## 4. Architecture

Components and responsibilities:

- Generator: argument parsing, validation, symbol encoding, the post-write header check and the viewer page. Owns argument semantics, output naming, stdout and stderr text, exit codes.
- Symbol encoder: module layout, mask selection, PNG raster. Owns module content and pixel output.
- `tools/gate.py`: black-box harness that runs the shipped artefact, decodes its output and asserts the contract. Stays in Python, is not shipped.
- `tools/argv_matrix.py`: runs the command-line cases in `spec/argv-matrix.json` against the shipped artefact or a target binary. The fixture is frozen from the Python line; the harness normalises the program name and scratch paths, collapses whitespace in stderr, and compares help stdout with whitespace collapsed because argparse wraps on the program name.
- `SKILL.md`: the text an agent reads; owns discovery, usage guidance, the platform-to-binary rule and the secrets rule.
- Viewer page: static HTML next to the PNG, no script, one image reference.

Data flow: argv -> validated payload -> symbol -> PNG on disk -> header check -> stdout paths -> optional page.

There is no persistent state, no configuration file and no service. Each run is independent.

Repository layout: the Python generator lives in `python\`, the gate in `tools\`, the crate in `rust-src\` from P1, the built binaries in `bin\`. `spec\`, `docs\` and `seed\` hold documents and history. `temp\` and `logs\` are untracked scratch. Nothing in the tree is shipped except `SKILL.md` and the generator.

The Python line is source of truth until the port lands. From the first Rust release the Python generator is deleted from the tree, git history is the only reference, and nothing reads or runs it any more. `seed/` keeps the original Desktop copies as history and is never a build input.

## 5. Interfaces

Command line:

| Flag | Type | Range or default | Notes |
|---|---|---|---|
| text | string | one argument | positional, optional |
| `-f`, `--file` | path | any readable file | UTF-8, stripped |
| `-o`, `--out` | path | default `qr.png` | relative to the current directory |
| `--scale` | int | 1-100, default 10 | pixels per module |
| `--border` | int | 0-16, default 4 | module units |
| `--html` | flag | off | writes the page |
| `--title` | string | default `qr code` | page only, never encoded |
| `--version` | flag | off | prints the version, added by the port |

Streams: stdout carries the absolute paths, one per line, and nothing else on success. stderr carries one line for a runtime failure, prefixed `FAIL`. Usage errors use the argparse-compatible text on stderr.

Files: the PNG is the only required output; the page is optional. The page path is the PNG path with the extension replaced by `.html`, so `qr.png` gives `qr.html` and a name without an extension gains `.html`.

### 5.1 Distribution

| Id | Requirement |
|---|---|
| D-01 | The installed skill folder holds `SKILL.md` and `bin\qr.exe` on Windows, `SKILL.md` and `bin/qr` on Linux. One folder, both binaries, no build step. |
| D-02 | `bin/qr` is built for `x86_64-unknown-linux-musl` and statically linked, so it runs on Alpine with no runtime package. |
| D-03 | `bin/qr.exe` is a console-subsystem binary, so a shell waits for it and captures stdout, stderr and the exit code. No GUI subsystem, no detached console. |
| D-04 | Both binaries are produced from one source tree and carry the same version, printed by `--version`. |
| D-05 | The built binaries are committed under `bin/` in this repo, because an install is a folder copy. A rebuild replaces them; sizes and digests go in the release notes, not in the tree. |
| D-06 | The gate and its decoder dependencies are not shipped and are not needed on the target. |

Breaking-change rule: flag names, defaults, ranges, message text, path derivation and exit codes are the contract. Removing a flag or narrowing a range is breaking; adding a flag with a default that preserves existing output is compatible. Replacing the shipped artefact from a script to a binary is breaking for installers, so the port releases as 0.8.0, one minor step above 0.7.0. The line stays below 1.0.0 until the skill has a stable install model and no planned breaking change.

## 6. Functionality and states

| State | Trigger | What the caller sees |
|---|---|---|
| Success, plain | valid payload, no `--html` | one absolute PNG path on stdout, exit 0 |
| Success, page | valid payload, `--html` | PNG path then page path, exit 0 |
| Version | `--version` | one version line on stdout, exit 0 |
| Usage error | conflicting or missing source, out-of-range number, unknown option | argparse-compatible text on stderr, exit 2, nothing written |
| Capacity failure | payload over level M capacity | one `FAIL store:` line on stderr, exit 1, no file |
| Size mismatch | header dimensions differ from the symbol size, or the file has no IEND chunk | one `FAIL png size` or `FAIL png tail` line on stderr, file removed, exit 1 |
| Filesystem failure | unwritable output path, unreadable or non-UTF-8 input | one `FAIL read` or `FAIL write` line on stderr, exit 1, no traceback |
| Page failure | the PNG is written and the page path is blocked | one `FAIL write` line on stderr, exit 1, PNG kept, stdout empty |

Recovery is always a corrected re-run; the tool holds no state that a failed run can corrupt.

## 7. Security and privacy

- Input trust: the payload is data. It is never passed to a shell, never interpolated into HTML, and never used as a path component. The only interpolation into HTML is the escaped title and the base name of the PNG.
- Filesystem reach: read the `-f` path, write the `-o` path and optionally the page path. Nothing else, no temp directory.
- Network reach: none, by NFR-01.
- Credentials: none used or stored.
- Secret lifecycle: a QR of a token is a photographable secret. The tool stores nothing, prints nothing of the payload, and the operator deletes the artefact after scanning, per the SKILL.md rule.
- Enforcement points: escaping for QR-16, base-name-only reference for QR-15, and the gate checks `viewer hides text` and `viewer references png` for both.
- Supply chain: the binary is built from this repo and committed here, so the installed artefact is the reviewed artefact. Crates are pinned in `Cargo.lock`.

## 8. Implementation constraints

- argparse behaviour stays part of the contract: prefix matching for long options, `-h` inside a short cluster, and exit 2 with the text on stderr.
- The post-write check reads the PNG header only. QR-13 is a header check, not a full decode; decode evidence comes from the gate.
- The Rust encoder must be pure Rust with no C dependency, so the musl build stays static and cross-compiles without a C toolchain for the target. The crate is chosen in P1 and recorded in the decision history.
- Port constraints, taken from `docs/python-to-rust-refactor-pitfalls.md`:
  - Applies: section 5 (one strict UTF-8 read, invalid input to `FAIL read`, no lossy path), section 6 (Result branches instead of `unwrap`), section 7 (release profile, `overflow-checks` if arithmetic is added), section 8 (the `-f` strip set only), section 10 (argv parity, path derivation, stdout encoding), section 11 (console subsystem choice), and section 12 (checklist and golden differential test).
  - Does not apply, with reason: section 1 (the program writes one file once, there is no loop of small writes), section 2 (no string building in a loop), section 3 (no per-iteration allocation in application code), section 4 (no bignum arithmetic), section 9 (no decode chain with error handlers).
  - A mechanical port must not add buffering, tables or decoders that the behaviour does not need.
- Kanji mode is not reproduced. segno selects it for text that Shift-JIS encodes; the port writes UTF-8 byte mode instead, so such a payload can need one or more extra versions. `spec/corpus.txt` excludes those payloads and the decision history records the difference.
- The port keeps the same message text and exit codes; deviations need a recorded decision.

## 9. Validation

| Requirement | Method | Evidence today |
|---|---|---|
| QR-01, QR-12 | gate decode | `decode matches`, `short payload regular 21 modules` |
| QR-02 | gate decode | `file exit 0`, `strips file whitespace`, `crlf payload preserved`, `strip set boundary` |
| QR-03 | gate exit code | `rejects text plus file`, `rejects empty text with file` |
| QR-04 | gate exit code | `rejects no source` |
| QR-05 | gate exit code | `rejects empty text` |
| QR-06 | gate stdout | `prints one absolute path` |
| QR-07 | gate file existence | `creates nested parents` |
| QR-08, QR-09 | gate exit code | `rejects scale 0`, `rejects border 17` |
| QR-10 | gate decode level and pixels | `short payload level M`, `url level M not boosted`, `short payload regular 21 modules` |
| QR-11 | gate exit code and file absence | `rejects oversize` |
| QR-13 | gate unit on a truncated file | `png tail detects truncation`; the size-mismatch branch is not injectable from outside and stays unproven |
| QR-14 | gate stdout | `two paths png then html` |
| QR-15 | gate page inspection | `viewer references png`, `viewer hides text`, `viewport tag present` |
| QR-16 | gate page inspection | `title escaped in page`, `title never encoded` |
| QR-17 | exit code asserted in every check | all 35 |
| QR-18 | gate decode after a rewrite | `overwrite replaces the file` |
| QR-19 | gate exit code and one-line stderr | `missing file is one FAIL read line`, `non utf8 file is one FAIL read line`, `blocks output parent with FAIL write`, `blocks page with FAIL write` |
| QR-20 | gate exit code | `help exits 0` |
| QR-21 | gate decode with stdin attached | `stdin is ignored` |
| QR-22 | not gated on the Python line | 0.8.0, port phase |
| NFR-01, NFR-02, NFR-07 | static inspection and repeat-run hash | gap |
| NFR-04, NFR-05, NFR-10 | measurement on the reference machine | gap, port phase |
| D-01 to D-03, NFR-09 | install into a scratch folder, `readelf` shows no `PT_INTERP` and no `NEEDED`, run on Alpine | gap, port phase |
| D-04, D-05 | release notes carry sizes and digests; both binaries report the same `--version` | gap, port phase |

Acceptance criteria for the port: the gate passes with the Rust binary in place of the script; a differential harness runs both implementations over the same fixture set and compares exit code, stdout, stderr, PNG payload decode and dimensions, and the page bytes. PNG bytes are not compared across implementations, because a different encoder produces a different valid file; QR-12 and NFR-07 cover what matters. Payloads where segno selects kanji mode are outside the dimension comparison, since the port writes them as UTF-8 bytes. The sweep runs on Windows and on Linux in `.github/workflows/ci.yml`, and the Linux binary also runs on Alpine x86_64 in that workflow.

## 10. Phases and gates

| Phase | State |
|---|---|
| Define | closed 2026-11-15: problem, goals, non-goals, requirements |
| Design | under review: architecture, interfaces, distribution, states, security, validation map |
| Build, Python 0.7.0 | shipped and gated: 15 of 15 |
| Build, Python 0.7.1 | gated: 35 of 35 |
| Build, port increments | not started |
| Release, 0.8.0 | not started |

Port increments, each closing at Gate 2:

1. P1: crate in `rust-src\`, encoder and CLI parity, no page. Gate: `tools\argv_matrix.py --target` matches the 27 frozen cases in `spec\argv-matrix.json` on exit code, stdout and stderr, and every fixture decodes to the same payload at the same dimensions.
2. P2: viewer page parity. Gate: the page is byte-equal to the Python output for the same title and PNG stem.
3. P3: packaging. One crate in `rust-src\` builds both targets: Windows console build and musl static build, committed under `bin\`; SKILL.md rewritten to call the binary per platform; Python generator deleted. Gate: install into an empty scratch folder, run both, `readelf` static proof, Alpine run, size and digest record.
4. P4: differential harness in the gate. Gate: the full fixture sweep is green on Windows and Linux, and the gate keeps its Python dependency set.

Gate 1 waits for approval of this sheet. OQ-1 to OQ-5 are closed.

## 11. Open questions

| Id | Question | Owner | Consequence if open |
|---|---|---|---|
| OQ-1 | Closed 2026-11-15: the skill ships per-platform binaries inside the qr folder, and the Rust line replaces Python, which is deleted from the tree and kept only as history. Alpine x86_64 is a target. | maintainer | none |
| OQ-2 | Closed 2026-11-15: after the port the version lives in `rust-src\Cargo.toml` alone, SKILL.md carries no second number, and `--version` prints the manifest value. | maintainer | none |
| OQ-3 | Closed 2026-11-15: stdout and stderr are UTF-8 on every platform. The ANSI codepage on Windows is a property of the Python line and is not reproduced; the differential harness compares message streams as text and file bytes exactly. | maintainer | none |
| OQ-4 | Closed 2026-11-15: the gate stays in Python on the development machine, is not shipped and does not run on Alpine. | maintainer | none |
| OQ-5 | Closed 2026-11-15: Alpine verification runs in GitHub Actions. The workflow checks `bin/qr` with `readelf` for a NEEDED entry and runs it in an `alpine:3.21` container, then replays the CLI matrix against the static binary. No local Docker or chroot is needed, and WSL stays a development environment for rustc. | maintainer | none |

## 12. Decision history

| Date | Decision | Reason |
|---|---|---|
| 2026-11-15 | Spec sheet written from the code, README and SKILL.md; status under review | the port needs a fixed behaviour target |
| 2026-11-15 | Error correction fixed at M, not selectable | one behaviour to test, one capacity limit to document |
| 2026-11-15 | `-f` decodes UTF-8 explicitly | platform default encoding must not change the payload |
| 2026-11-15 | The page carries no payload and no script | a photographed screen must not expose text, and a static file needs no script |
| 2026-11-15 | Header check on write, not a full decode | keeps the generator free of Pillow and zxing-cpp |
| 2026-11-15 | Per-platform binaries ship inside the skill folder: `bin\qr.exe` and `bin/qr` | install stays a folder copy, no build step on the target |
| 2026-11-15 | Linux builds for `x86_64-unknown-linux-musl`, statically linked | Alpine and any glibc-free image must run it |
| 2026-11-15 | The Rust line replaces Python; the Python generator is deleted at the port and kept only as history | one source of truth, no dual maintenance |
| 2026-11-15 | Version 0.8.0 for the port, 0.7.0 for the Python line | the port changes the shipped artefact, and the skill is not at a stable 1.0.0 install model yet |
| 2026-11-15 | Console subsystem on Windows | a shell must wait for the exit code and capture both streams |
| 2026-11-15 | PNG compared by decode and dimensions, not by hash | two encoders produce different bytes for the same symbol |
| 2026-11-15 | One folder per language: `python\`, `rust-src\`, with tools in `tools\`; `temp\` and `logs\` ignored | the root stays readable and each language has one home |
| 2026-11-15 | For 0.7.0 the version home is `metadata.version` in SKILL.md, because the line has no build manifest; from 0.8.0 the manifest rule applies | the manual asks for one home, and a script has no manifest yet |
| 2026-11-15 | Regular QR only; Micro QR is never emitted | scanner support varies, and the common pure-Rust encoders do not emit Micro QR, so one symbol family keeps parity |
| 2026-11-15 | The `-f` strip set is CPython's measured 10-range set, carried into the port as a table, not replaced by Rust `trim` | a patch release must not change the payload, and `trim` keeps U+001C to U+001F that CPython strips |
| 2026-11-15 | 0.7.1 closes the audit findings and the contract now names the IEND check, the `FAIL read` and `FAIL write` lines, the page-failure state and the HTML-escaped title | the audit found the documented behaviour and the code apart, and the fixes need a written target |
| 2026-11-15 | After the port the version lives in the crate manifest alone, and `--version` prints it | two homes drift, and the binary can report the number it was built from |
| 2026-11-15 | stdout and stderr are UTF-8 on both platforms, and the Python ANSI-codepage behaviour is not reproduced | paths and messages decode as text, and the harness compares text rather than bytes |
| 2026-11-15 | The gate stays a Python tool on the development machine and never ships | decoding needs zxing-cpp, and the target has no interpreter after the port |
| 2026-11-15 | Alpine verification moves into GitHub Actions against the committed binary: `readelf` for no NEEDED entry, then a run inside `alpine:3.21`, then the CLI matrix | the runner already has Docker, no local Docker or chroot is needed, and the check is reproducible for anyone with repo access |
| 2026-11-15 | Kanji mode is not reproduced: Shift-JIS text is written as UTF-8 byte mode, which can add versions | a kanji segment needs a Shift-JIS table for a niche path, and the byte-mode symbol decodes to the same text |
| 2026-11-15 | Decoding, other symbologies and image input wait for 0.8.0 and the public switch, then ship as separate crates | the port must land first, and each new capability carries its own dependency and review surface |
