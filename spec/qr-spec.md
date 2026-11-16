# QR skill spec sheet

| Field | Value |
|---|---|
| Title | qr: offline QR code generator skill |
| Purpose | Pin observable behaviour of the skill, first for the Python line, then as the acceptance target for the Rust line that replaces it |
| Status | under review |
| Version | 1.1.0 |
| Owner | skill maintainer |
| Stakeholders | agent operators who load the skill, reviewers of the port |
| Approvers | owner, reviewer |
| Related | agent-skill/SKILL.md, README.md, tools/gate.py, tools/argv_matrix.py, spec/argv-matrix.json, CHANGELOG.md, docs/specsheet-design-and-review-manual.md, docs/audit-process-manual.md, docs/changelog-build-and-release-manual.md, docs/python-to-rust-refactor-pitfalls.md |
| Scope | the shipped skill, SKILL.md and the generator; the gate is verification, not product |

## 1. Problem and goals

An agent that needs a scannable code for a URL, token, Wi-Fi string or contact card has no dependable way to produce one. Ad hoc attempts write throwaway scripts, call an online generator, or leave images in a temp folder that outlives the request. For a token or password payload a network round trip is a disclosure, and a code drawn without a quiet zone or at one pixel per module fails on many scanners.

Goals:

- G1: one command turns short text into a PNG that decodes back to the exact input.
- G2: an optional viewer page shows the code large enough to scan from a screen.
- G3: a failed run never prints a path to a broken or absent file.
- G4: install is a folder copy with no interpreter, package manager or build step on the target.
- G5: a code that only has to be looked at is shown without writing any file.

Non-goals: decoding, batch generation, SVG or EPS output, logos or branding, animated codes, a choice of error-correction level, network delivery, a GUI, Intel macOS builds. Decoding, other symbologies and image input are planned as separate crates after the public switch.

## 2. Functional requirements

QR-22 applies to the 0.8.0 binary; the 0.7.1 Python line had no `--version` flag. QR-23 and QR-24 apply from 0.10.0; 0.9.0 carried the same terminal rendering behind an opt-in `--term` flag, which 0.10.0 removed.

| Id | Required behaviour | Failure behaviour |
|---|---|---|
| QR-01 | A single positional argument is encoded unchanged: no strip, no case change, no re-encoding. | none |
| QR-02 | `-f PATH` reads the file as UTF-8, without line-ending translation, and strips leading and trailing whitespace before encoding. The whitespace set is CPython's, measured on 3.14.6: U+0009 to U+000D, U+001C to U+0020, U+0085, U+00A0, U+1680, U+2000 to U+200A, U+2028, U+2029, U+202F, U+205F, U+3000, 29 code points in 10 ranges. Rust `str::trim` follows Unicode White_Space and keeps U+001C to U+001F, so the port carries this table. | QR-19 |
| QR-03 | Text and `-f` are mutually exclusive. | exit 2, stderr `give text or --file, not both`, nothing written |
| QR-04 | Neither text nor `-f` is given. | exit 2, stderr `provide text or --file`, nothing written |
| QR-05 | The payload is empty, either as an empty argument or a file that strips to nothing. | exit 2, stderr `empty input`, nothing written |
| QR-06 | `-o PATH` sets the PNG path, default `qr.png`, resolved against the current directory; the absolute path is printed. It selects save mode, per QR-24. | QR-19 |
| QR-07 | Missing parent directories of the output path are created, including nested ones. | QR-19 |
| QR-08 | `--scale` is an integer 1 to 100, default 10 pixels per module. It selects save mode, per QR-24. | exit 2, stderr `--scale must be 1-100` |
| QR-09 | `--border` is an integer 0 to 16, default 4 quiet-zone modules. | exit 2, stderr `--border must be 0-16` |
| QR-10 | Error correction is fixed at level M and is not selectable. Symbols are regular QR, never Micro QR. | no option exists |
| QR-11 | A payload over the capacity of level M fails without leaving a file. | exit 1, stderr starts `FAIL store:` |
| QR-12 | The PNG is square, side `(modules + 2 * border) * scale` pixels, and decodes back to the payload. | QR-11, QR-13 |
| QR-13 | After writing, the PNG header dimensions are compared with the expected symbol size, and the file must end with an IEND chunk. | on a size mismatch the file is removed, exit 1, stderr `FAIL png size <got> expected <want>`; without IEND the file is removed, exit 1, stderr `FAIL png tail: missing IEND` |
| QR-14 | In save mode, stdout carries only the absolute PNG path, then, with `--html`, the absolute page path on a second line. | no other stdout output in save mode |
| QR-15 | `--html` writes a page named after the PNG stem, next to the PNG, referencing the image by file name; it contains no payload text, no script and no external asset; the image is scaled to 70vmin, the shorter side of the viewport. It selects save mode, per QR-24. | QR-19 |
| QR-16 | `--title` sets the page title and caption only, HTML-escaped, and is never encoded. It selects save mode, per QR-24. | none; any string is accepted |
| QR-17 | Exit 0 on success, 1 on a runtime failure, 2 on a usage error. | n/a |
| QR-18 | An existing output file is replaced. | QR-19 |
| QR-19 | Any filesystem failure, including an unreadable, missing or non-UTF-8 `-f` file, ends with a one-line error on stderr and exit 1, with no traceback and no partial file left behind. | n/a |
| QR-20 | `-h` and `--help` print usage on stdout and exit 0; the text lists every option, `--version` included, in the option list and the usage line. An unknown option is a usage error. | exit 2 for unknown options |
| QR-21 | Standard input is never read. | n/a |
| QR-22 | `--version` prints the single version string and exits 0, on every build. | exit 0 always |
| QR-23 | With no output option the symbol is printed to stdout as UTF-8 text, quiet zone included, black modules on a painted light background, two module rows per text row. It writes no file, creates no directory and prints no path. | a failed stdout write is one `FAIL write: stdout` line and exit 1 |
| QR-24 | `--save` selects save mode and writes the PNG at `-o`, default `qr.png`. `-o`, `--scale`, `--html` and `--title` also select save mode; `-f` and `--border` do not. | QR-19 |

## 3. Non-functional requirements

| Id | Requirement |
|---|---|
| NFR-01 | No network at runtime: no socket, DNS or HTTP. |
| NFR-02 | The generator needs no interpreter or package at runtime. The gate's dependencies stay in the gate. |
| NFR-03 | Repository text is ASCII with LF endings; generated images and gate scratch are not committed. |
| NFR-04 | One PNG of a URL of 60 characters completes in under 1 s on the reference machine. Measured: 10.3 ms maximum over five runs. |
| NFR-05 | Peak memory stays under 16 MiB for a 2000-byte payload. Measured: 0.97 MiB peak commit (job object) at the default scale. |
| NFR-06 | The five builds behave identically for file output, exit codes and file names. |
| NFR-07 | The same payload and options produce byte-identical PNG output on the same platform. |
| NFR-08 | The payload never reaches stdout, stderr, a log or a file name, and no telemetry is collected. |
| NFR-09 | Both Linux binaries are statically linked and carry no dynamic loader dependency; they start on Alpine x86_64 and arm64 with no package installed. |
| NFR-10 | The Windows and macOS binaries start with no runtime redistributable. Size under 4 MB per binary. Measured on the 1.1.0 artifacts: 249856 bytes for `qr.exe`, 235008 for `qr-arm64.exe`, 656984 for `qr`, 715912 for `qr-arm64`, 535888 for `qr-macos-arm64`. |
| NFR-11 | The build uses one source tree and one version for all five binaries, with no per-platform source differences. |
| NFR-12 | A run with no output option performs no filesystem write. After a terminal run the working directory is unchanged, with no PNG, page, temp file, cache or log. |

## 4. Architecture

Components and responsibilities:

- Generator: argument parsing, validation, symbol encoding, the post-write header check and the viewer page. Owns argument semantics, output naming, stdout and stderr text, exit codes.
- Symbol encoder: module layout, mask selection, PNG raster. Owns module content and pixel output.
- `tools/gate.py`: black-box harness that runs the shipped artefact, decodes its output and asserts the contract. Stays in Python, is not shipped.
- `tools/argv_matrix.py`: runs the command-line cases in `spec/argv-matrix.json` against the shipped artefact or a target binary. The fixture is frozen from the binary; the harness normalises the program name and scratch paths, collapses whitespace in stderr, and compares help stdout with whitespace collapsed because the usage line wraps on the program name.
- `tools/diff_python.py`: differential sweep over the corpus and a page set, comparing a binary with the Python line. Run on Windows and Linux before deletion, kept for a re-run against a script taken from git history.
- `SKILL.md`: the text an agent reads; owns discovery, usage guidance, the platform-to-binary rule and the secrets rule.
- Viewer page: static HTML next to the PNG, no script, one image reference.
- Terminal renderer: `rust-src/src/term.rs` owns the block glyphs, the painted background, the two-rows-per-cell pairing and the quiet zone. It turns the encoded symbol into a string and has no filesystem access.

Data flow: argv -> validated payload -> symbol -> painted text on stdout by default; with save mode, symbol -> PNG on disk -> header check -> stdout paths -> optional page.

There is no persistent state, no configuration file and no service. Each run is independent.

Repository layout: the skill payload lives in `agent-skill\` with `SKILL.md` and `bin\`; the crate lives in `rust-src\`, the gate in `tools\`, and `spec\` and `docs\` hold documents and history. `temp\`, `logs\` and `releases\` are untracked scratch. Nothing in the tree is shipped except the `agent-skill\` folder.

The Python line was the source of truth until 0.8.0. The generator is deleted from the tree, git history holds the only copies, and the product does not run it.

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
| `--title` | string | default `qr code` | page only, never encoded; implies save |
| `--save` | flag | off | writes the PNG instead of printing to the terminal |
| `--version` | flag | off | prints the version, added by the port |

Streams: by default stdout carries the painted symbol and no path. In save mode stdout carries the absolute paths, one per line, and nothing else on success. stderr carries one line for a runtime failure, prefixed `FAIL`. Usage errors use the argparse-compatible text on stderr.

Files: in save mode the PNG is the only required output and the page is optional; a default run writes no file. The output is always written as PNG, whatever the extension. The page path is the PNG path with the extension replaced by `.html`, so `qr.png` gives `qr.html` and a name without an extension gains `.html`.

### 5.1 Distribution

| Id | Requirement |
|---|---|
| D-01 | The installed skill folder holds `SKILL.md` and five binaries: `bin\qr.exe` and `bin\qr-arm64.exe` for Windows, `bin/qr` and `bin/qr-arm64` for Linux, and `bin/qr-macos-arm64` for macOS ARM64. One folder, no build step. `SKILL.md` names the binary for the host architecture; Intel macOS is not shipped. |
| D-02 | `bin/qr` and `bin/qr-arm64` are built for `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, statically linked, so they run on Alpine with no runtime package. |
| D-03 | The two Windows binaries use the console subsystem, so a shell waits for them and captures stdout, stderr and the exit code. No GUI subsystem, no detached console. |
| D-04 | All five binaries are produced from one source tree and carry the same version, printed by `--version`. |
| D-05 | The built binaries are committed under `agent-skill/bin/` in this repo, because an install is a folder copy. A rebuild replaces them; sizes and digests go in the release notes, not in the tree. |
| D-06 | The gate and its decoder dependencies are not shipped and are not needed on the target. |

Breaking-change rule: flag names, defaults, ranges, message text, path derivation and exit codes are the contract. Removing a flag or narrowing a range is breaking; adding a flag with a default that preserves existing output is compatible. Replacing the shipped artefact from a script to a binary is breaking for installers, so the port releases as 0.8.0, one minor step above 0.7.0. 1.0.0 marks the stable install model: native binaries for every shipped platform in one folder, one command line, no planned breaking change.

## 6. Functionality and states

| State | Trigger | What the caller sees |
|---|---|---|
| Success, plain | valid payload, no `--html` | one absolute PNG path on stdout, exit 0 |
| Success, page | valid payload, `--html` | PNG path then page path, exit 0 |
| Success, terminal | valid payload, no output option | painted symbol on stdout, exit 0, no file written |
| Success, page | valid payload, `--html` | PNG path then page path, exit 0 |
| Version | `--version` | one version line on stdout, exit 0 |
| Usage error | conflicting or missing source, out-of-range number, unknown option | argparse-compatible text on stderr, exit 2, nothing written |
| Capacity failure | payload over level M capacity | one `FAIL store:` line on stderr, exit 1, no file |
| Size mismatch | header dimensions differ from the symbol size, or the file has no IEND chunk | one `FAIL png size` or `FAIL png tail` line on stderr, file removed, exit 1 |
| Filesystem failure | unwritable output path, unreadable or non-UTF-8 input | one `FAIL read` or `FAIL write` line on stderr, exit 1, no traceback |
| Page failure | the PNG is written and the page path is blocked | one `FAIL write` line on stderr, exit 1, PNG kept, stdout empty |
| Terminal write failure | stdout is closed or unwritable during `--term` | one `FAIL write: stdout` line on stderr, exit 1 |

Recovery is always a corrected re-run; the tool holds no state that a failed run can corrupt.

## 7. Security and privacy

- Input trust: the payload is data. It is never passed to a shell, never interpolated into HTML, and never used as a path component. The only interpolation into HTML is the escaped title and the base name of the PNG.
- Filesystem reach: read the `-f` path; in save mode write the `-o` path and optionally the page path. Nothing else, no temp directory. A default run writes nothing.
- Network reach: none, by NFR-01.
- Credentials: none used or stored.
- Secret lifecycle: a QR of a token is a photographable secret. The tool stores nothing, prints nothing of the payload, and the operator deletes the artefact after scanning, per the SKILL.md rule. The default terminal mode leaves no artefact to delete and is the preferred mode for a one-off code.
- Enforcement points: escaping for QR-16, base-name-only reference for QR-15, and the gate checks `viewer hides text` and `viewer references png` on the shipped binary.
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
| QR-13 | unit tests on the probe and the verdict, gate unit on a truncated file | `verify_rejects_a_size_mismatch`, `verify_names_the_size_and_the_tail`; the gate covers truncation with `png tail detects truncation` |
| QR-14 | gate stdout | `two paths png then html` |
| QR-15 | gate page inspection | `viewer references png`, `viewer hides text`, `viewport tag present` |
| QR-16 | gate page inspection | `title escaped in page`, `title never encoded` |
| QR-17 | exit code asserted in every check | all 45 |
| QR-18 | gate decode after a rewrite | `overwrite replaces the file` |
| QR-19 | gate exit code and one-line stderr | `missing file is one FAIL read line`, `non utf8 file is one FAIL read line`, `blocks output parent with FAIL write`, `blocks page with FAIL write` |
| QR-20 | gate exit code and matrix help cases | `help exits 0`; the matrix compares the help text after whitespace collapse |
| QR-21 | gate decode with stdin attached | `stdin is ignored` |
| QR-22 | gate exit code and stdout | `version prints one line` |
| QR-23, QR-24, NFR-12 | gate terminal rebuild, directory listing and save default | `term exits 0 with no stderr`, `term writes nothing to disk`, `term decodes to the payload`, `term keeps a square white quiet zone`, `term border 0 gives a bare 21-module grid`, `term oversize writes nothing`, `save writes qr.png and prints the path`, `save writes a decodable PNG` |
| NFR-01, NFR-02, NFR-07 | static inspection and repeat-run hash | no network code in the tree; one static binary with no runtime dependency; two runs of the same payload give the same PNG digest |
| NFR-04, NFR-05, NFR-10 | measurement on the reference machine | 10.3 ms maximum, 0.97 MiB peak commit, 249856, 235008, 656984, 715912 and 535888 bytes |
| D-01 to D-03, NFR-09 | install into a scratch folder, `readelf` shows no `NEEDED`, run on Alpine | scratch copy of `SKILL.md` and `bin` runs and prints both paths; the readelf and `alpine:3.21` jobs are green for both Linux builds, and the five binaries pass the gate on their own runners, macOS included (run 43805428587) |
| D-04, D-05 | release notes carry sizes and digests; the five binaries report the same `--version` | 1.1.0: all five print 1.1.0; v1.1.0 is tagged on 3cdd391 and published with five archives, 130178, 126596, 299016, 297308 and 229378 bytes, each digest in the notes; the assets were re-downloaded and hash-compared, and the extracted Windows binary passed the gate 45 of 45 and the matrix 30 of 30 |

Acceptance criteria for the port, and the evidence: the gate passes with the Rust binary in place of the script (36 of 36 on both builds); the CLI matrix matches 27 of 27 on both; the differential sweep compares exit code, printed names, payload decode, dimensions and page bytes over 91 payloads and 9 pages on Windows and Linux, with PNG bytes not compared across implementations because a different encoder produces a different valid file (QR-12 and NFR-07 cover what matters). Payloads where segno selects kanji mode are outside the dimension comparison, since the port writes them as UTF-8 bytes. The sweep ran before the Python line was deleted (run 43732908549); `tools/diff_python.py` stays for a re-run against a script taken from history, and the permanent checks are the gate, the matrix and symbol parity. The Linux binary also runs on Alpine x86_64 in that workflow.

## 10. Phases and gates

| Phase | State |
|---|---|
| Define | closed 2026-11-15: problem, goals, non-goals, requirements |
| Design | under review: architecture, interfaces, distribution, states, security, validation map |
| Build, Python 0.7.0 | shipped and gated: 15 of 15 |
| Build, Python 0.7.1 | gated: 35 of 35 |
| Build, port increments | P1 to P4 closed |
| Build, 0.9.0 | P5: terminal display behind `--term`, 47 checks on both platforms |
| Build, 0.10.0 | P6: terminal display by default, save mode behind `--save` and the file options |
| Build, 0.10.1 | help lists `--version`; unit tests cover the post-write size and tail verdict |
| Build, 1.0.0 | ARM64 on both systems: aarch64 Linux musl and Windows ARM64, built on the arm64 runners; four binaries at 1.0.0 |
| Build, 1.1.0 | macOS ARM64: aarch64-apple-darwin built on macos-latest, no Intel macOS binary |
| Release, 0.8.0 | shipped 2026-11-15: v0.8.0 tagged on b976acf, two archives published, assets re-verified at the release page |
| Release, 0.9.0 | shipped 2026-11-15: v0.9.0 tagged on 7994592, two archives published, assets re-verified at the release page |
| Release, 0.10.0 | shipped 2026-11-16: v0.10.0 tagged on fecf43d, two archives published, assets re-verified at the release page |
| Release, 0.10.1 | shipped 2026-11-16: v0.10.1 tagged on 7517fd1, two archives published, assets re-verified at the release page |
| Release, 1.0.0 | shipped 2026-11-16: v1.0.0 tagged on a9be694, four archives published, assets re-verified at the release page |
| Release, 1.1.0 | shipped 2026-11-16: v1.1.0 tagged on 3cdd391, five archives published, assets re-verified at the release page |

Port increments, each closing at Gate 2:

1. P1: crate in `rust-src\`, encoder and CLI parity, no page. Gate met: `tools\argv_matrix.py` 27 of 27 on both binaries and the gate passes 36 of 36 on the built binary.
2. P2: viewer page parity. Gate met: page bytes equal over nine titles and names, including escaping, an empty title and control characters.
3. P3: packaging. Gate met: both binaries committed under `bin\`, a scratch folder with `SKILL.md` and `bin` runs, `readelf` shows no `NEEDED`, the `alpine:3.21` job runs the static binary, sizes are 244736 and 648296 bytes, and the Python generator is deleted.
4. P4: differential harness. Gate met by the pre-deletion sweep: 91 payloads and 9 pages on Windows and Linux, with no difference in exit code, printed names, decoded text, dimensions or page bytes. The tool stays in `tools\`; it is not a permanent CI job because the reference is gone.
5. P5: terminal display. Gate met by the 47 checks, which rebuild the printed blocks into a bitmap, decode the payload and require an empty working directory after the run. 0.9.0 shipped this behind `--term`.
6. P6: terminal display by default. Gate met by 45 checks and 30 matrix cases: a run with no output option leaves the working directory empty, and `--save` writes a decodable PNG.

Gate 1 is closed; OQ-1 to OQ-5 are closed.

## 11. Open questions

| Id | Question | Owner | Consequence if open |
|---|---|---|---|
| OQ-1 | Closed 2026-11-15: the skill ships per-platform binaries inside the qr folder, and the Rust line replaces Python, which is deleted from the tree and kept only as history. Alpine x86_64 is a target. | maintainer | none |
| OQ-2 | Closed 2026-11-15: after the port the version lives in `rust-src\Cargo.toml` alone, SKILL.md carries no second number, and `--version` prints the manifest value. | maintainer | none |
| OQ-3 | Closed 2026-11-15: stdout and stderr are UTF-8 on every platform. The ANSI codepage on Windows is a property of the Python line and is not reproduced; the differential harness compares message streams as text and file bytes exactly. | maintainer | none |
| OQ-4 | Closed 2026-11-15: the gate stays in Python on the development machine, is not shipped and does not run on Alpine. | maintainer | none |
| OQ-5 | Closed 2026-11-15: Alpine verification runs in GitHub Actions. The workflow checks `bin/qr` with `readelf` for a NEEDED entry and runs it in an `alpine:3.21` container, then replays the CLI matrix against the static binary. No local Docker, Rust toolchain or WSL is needed. | maintainer | none |

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
| 2026-11-15 | The help text stays byte-equal to the 0.7.1 fixture, so `--version` is not listed; `SKILL.md` documents it and the gate checks it | the fixture is the CLI contract frozen while the Python line existed, and it cannot carry a flag the Python line lacks |
| 2026-11-15 | The gate tests `bin\qr.exe` or `bin/qr` by default and carries its own PNG probe | the Python module it used is deleted at 0.8.0, and the gate must exercise the shipped artefact |
| 2026-11-15 | The differential sweep runs once, before the Python line is deleted; CI keeps the frozen matrix, the gate and symbol parity | a deleted reference in CI would need segno and a copy of the old code on every run |
| 2026-11-15 | `-o` no longer requires a `.png` extension; the writer always emits PNG | the format no longer comes from an extension lookup, and refusing a path over its extension has no user value |
| 2026-11-15 | The release attaches two platform archives, each holding `SKILL.md`, `CHANGELOG.md`, `LICENSE` and one binary, rather than the binaries alone | the install unit is a folder copy, and the release manual rules out bare binaries |
| 2026-11-15 | 0.9.0 put the terminal display behind `--term`; 0.10.0 made it the default and replaced that flag with `--save` | the main use is a code scanned once, so the mode that cannot leave a file must need no flag; the file options already announce a file |
| 2026-11-15 | Terminal rendering uses half blocks on a painted bright-white background, two module rows per text row | a module stays square in a terminal cell, and a dark terminal still shows the dark-on-light polarity a scanner expects |
| 2026-11-16 | 0.10.1 lists `--version` in the help text and the usage line | the fixture is frozen from the Rust binary now, so the 0.7.1 text no longer constrains the help |
| 2026-11-16 | 1.0.0 adds ARM64 (`aarch64-unknown-linux-musl` and Windows ARM64) and marks the stable install model | the arm64 runners are GA in public repositories, the gate dependencies publish arm64 wheels, and the code has no architecture-specific part |
| 2026-11-16 | 1.1.0 adds macOS ARM64 and leaves Intel macOS to a source build | macos-latest is an arm64 runner, the gate dependencies publish macOS arm64 wheels, and the code has no target-specific part |
| 2026-11-16 | The skill payload moves to `agent-skill/`, holding `SKILL.md` and `bin/`; the crate, tools, spec and docs stay at the root | the root stays readable, and the install unit is one visible folder |
