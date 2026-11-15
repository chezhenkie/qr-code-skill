# QR skill spec sheet

| Field | Value |
|---|---|
| Title | qr: offline QR code generator skill |
| Purpose | Pin observable behaviour of the skill, first for the Python line, then as the acceptance target for a Rust port |
| Status | under review |
| Version | 1.0.0 |
| Owner | skill maintainer |
| Stakeholders | agent operators who load the skill, reviewers of the port |
| Approvers | owner, reviewer |
| Related | SKILL.md, README.md, gate/gate.py, docs/specsheet-design-and-review-manual.md, docs/audit-process-manual.md, docs/changelog-build-and-release-manual.md, docs/python-to-rust-refactor-pitfalls.md |
| Scope | the two shipped files, SKILL.md and make-qr.py; the gate is verification, not product |

## 1. Problem and goals

An agent that needs a scannable code for a URL, token, Wi-Fi string or contact card has no dependable way to produce one. Ad hoc attempts write throwaway scripts, call an online generator, or leave images in a temp folder that outlives the request. For a token or password payload a network round trip is a disclosure, and a code drawn without a quiet zone or at one pixel per module fails on many scanners.

Goals:

- G1: one command turns short text into a PNG that decodes back to the exact input.
- G2: an optional viewer page shows the code large enough to scan from a screen.
- G3: a failed run never prints a path to a broken or absent file.
- G4: install is a copy of two files plus one pure-Python dependency.

Non-goals: decoding, batch generation, SVG or EPS output, logos or branding, animated codes, a choice of error-correction level, network delivery, a GUI.

## 2. Functional requirements

| Id | Required behaviour | Failure behaviour |
|---|---|---|
| QR-01 | A single positional argument is encoded unchanged: no strip, no case change, no re-encoding. | none |
| QR-02 | `-f PATH` reads the file as UTF-8 and strips leading and trailing whitespace before encoding. | QR-19 |
| QR-03 | Text and `-f` are mutually exclusive. | exit 2, stderr `give text or --file, not both`, nothing written |
| QR-04 | Neither text nor `-f` is given. | exit 2, stderr `provide text or --file`, nothing written |
| QR-05 | The payload is empty, either as an empty argument or a file that strips to nothing. | exit 2, stderr `empty input`, nothing written |
| QR-06 | `-o PATH` sets the PNG path, default `qr.png`, resolved against the current directory; the absolute path is printed. | QR-19 |
| QR-07 | Missing parent directories of the output path are created, including nested ones. | QR-19 |
| QR-08 | `--scale` is an integer 1 to 100, default 10 pixels per module. | exit 2, stderr `--scale must be 1-100` |
| QR-09 | `--border` is an integer 0 to 16, default 4 quiet-zone modules. | exit 2, stderr `--border must be 0-16` |
| QR-10 | Error correction is fixed at level M and is not selectable. | no option exists |
| QR-11 | A payload over the capacity of level M fails without leaving a file. | exit 1, stderr starts `FAIL store:` |
| QR-12 | The PNG is square, side `(modules + 2 * border) * scale` pixels, and decodes back to the payload. | QR-11, QR-13 |
| QR-13 | After writing, the PNG header dimensions are compared with the expected symbol size. | on mismatch the file is removed, exit 1, stderr `FAIL png size <got> expected <want>` |
| QR-14 | stdout carries only the absolute PNG path, then, with `--html`, the absolute page path on a second line. | no other stdout output on any path |
| QR-15 | `--html` writes a page named after the PNG stem, next to the PNG, referencing the image by file name; it contains no payload text, no script and no external asset; the image is scaled to 70% of the smaller viewport side. | QR-19 |
| QR-16 | `--title` sets the page title and caption only, HTML-escaped, and is never encoded. | none; any string is accepted |
| QR-17 | Exit 0 on success, 1 on a runtime failure, 2 on a usage error. | n/a |
| QR-18 | An existing output file is replaced. | QR-19 |
| QR-19 | Any filesystem failure, including an unreadable, missing or non-UTF-8 `-f` file, ends with a one-line error on stderr and exit 1, with no traceback and no partial file left behind. | n/a |
| QR-20 | `-h` and `--help` print usage on stdout and exit 0; an unknown option is a usage error. | exit 2 for unknown options |
| QR-21 | Standard input is never read. | n/a |

## 3. Non-functional requirements

| Id | Requirement |
|---|---|
| NFR-01 | No network at runtime: no socket, DNS or HTTP, and the dependency set is pure Python. |
| NFR-02 | The generator imports segno and the standard library only. Pillow and zxing-cpp belong to the gate. |
| NFR-03 | Repository text is ASCII with LF endings; generated images and gate scratch are not committed. |
| NFR-04 | One PNG of a URL of 60 characters completes in under 1 s on the reference machine. Guess, to be measured at the port. |
| NFR-05 | Peak memory stays under 16 MiB for a 2000-byte payload. Guess, to be measured. |
| NFR-06 | Windows and Linux on CPython 3.14 behave identically for file output, exit codes and file names. |
| NFR-07 | The same payload and options produce byte-identical PNG output on the same platform. |
| NFR-08 | The payload never reaches stdout, stderr, a log or a file name, and no telemetry is collected. |

## 4. Architecture

Components and responsibilities:

- `make-qr.py`: argument parsing, validation, the segno call, the post-write header check and the viewer page. Owns argument semantics, output naming, stdout and stderr text, exit codes.
- `segno`: symbol layout, mask selection and PNG encoding. Owns module content and pixel raster.
- `gate/gate.py`: black-box harness that runs the shipped script, decodes its output and asserts the contract.
- `SKILL.md`: the text an agent reads; owns discovery, usage guidance and the secrets rule.
- Viewer page: static HTML next to the PNG, no script, one image reference.

Data flow: argv -> validated payload -> segno symbol -> PNG on disk -> header check -> stdout paths -> optional page.

There is no persistent state, no configuration file and no service. Each run is independent.

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

Streams: stdout carries the absolute paths, one per line, and nothing else on success. stderr carries one line for a runtime failure, prefixed `FAIL`. Usage errors use the argparse text on stderr.

Files: the PNG is the only required output; the page is optional. The page path is the PNG path with the extension replaced by `.html`, so `qr.png` gives `qr.html` and a name without an extension gains `.html`.

Breaking-change rule: flag names, defaults, ranges, message text, path derivation and exit codes are the contract. Removing a flag or narrowing a range is breaking; adding a flag with a default that preserves existing output is compatible.

## 6. Functionality and states

| State | Trigger | What the caller sees |
|---|---|---|
| Success, plain | valid payload, no `--html` | one absolute PNG path on stdout, exit 0 |
| Success, page | valid payload, `--html` | PNG path then page path, exit 0 |
| Usage error | conflicting or missing source, out-of-range number, unknown option | argparse text on stderr, exit 2, nothing written |
| Capacity failure | payload over level M capacity | one `FAIL store:` line on stderr, exit 1, no file |
| Size mismatch | header dimensions differ from the symbol size | one `FAIL png size` line on stderr, file removed, exit 1 |
| Filesystem failure | unwritable path, unreadable input | one error line on stderr, exit 1, no partial output |

Recovery is always a corrected re-run; the tool holds no state that a failed run can corrupt.

## 7. Security and privacy

- Input trust: the payload is data. It is never passed to a shell, never interpolated into HTML, and never used as a path component. The only interpolation into HTML is the escaped title and the base name of the PNG.
- Filesystem reach: read the `-f` path, write the `-o` path and optionally the page path. Nothing else, no temp directory.
- Network reach: none, by NFR-01.
- Credentials: none used or stored.
- Secret lifecycle: a QR of a token is a photographable secret. The tool stores nothing, prints nothing of the payload, and the operator deletes the artifact after scanning, per the SKILL.md rule.
- Enforcement points: escaping in `make-qr.py` for QR-16, base-name-only reference for QR-15, and the gate checks `viewer hides text` and `viewer references png` for both.

## 8. Implementation constraints

- argparse behaviour stays part of the contract: prefix matching for long options, `-h` inside a short cluster, and exit 2 with the message on stderr.
- The post-write check reads the PNG header only. QR-13 is therefore a header check, not a full decode; decode evidence comes from the gate.
- Port constraints, taken from `docs/python-to-rust-refactor-pitfalls.md`:
  - Applies: section 6 (Result branches instead of `unwrap`), section 7 (release profile, `overflow-checks` if any arithmetic is added), section 10 (argv parity, path derivation, stdout encoding), section 11 (console subsystem choice), and section 12 (checklist and golden differential test).
  - Does not apply, with reason: section 1 (the program writes one file once, there is no loop of small writes), section 2 (no string building in a loop), section 3 (no per-iteration allocation in application code), section 4 (no bignum arithmetic), section 5 (UTF-8 handling is one decode at the input boundary), section 8 (no character classes, regex or line splitting), section 9 (no decode chain with error handlers).
  - A mechanical port must not add buffering, tables or decoders that the behaviour does not need.
- The port keeps the same stdout and stderr text and the same exit codes; deviations need a recorded decision.

## 9. Validation

| Requirement | Method | Evidence today |
|---|---|---|
| QR-01, QR-12 | gate decode | `decode matches` |
| QR-02 | gate decode | `file exit 0`, `strips file whitespace` |
| QR-03 | gate exit code | `rejects text plus file` |
| QR-05 | gate exit code | `rejects empty text` |
| QR-06 | gate stdout | `prints one absolute path` |
| QR-08, QR-09 | gate exit code | `rejects scale 0`, `rejects border 17` |
| QR-11 | gate exit code and file absence | `rejects oversize` |
| QR-14 | gate stdout | `two paths png then html` |
| QR-15 | gate page inspection | `viewer references png`, `viewer hides text` |
| QR-12 at two scales | gate pixels | `scale 12 pixels`, `default scale 10 pixels` |
| QR-04, QR-07, QR-10, QR-13, QR-16, QR-17, QR-18, QR-19, QR-20, QR-21 | not yet gated | gap, to be closed by the audit plan |
| NFR-01, NFR-02, NFR-07 | static inspection and repeat-run hash | gap |
| NFR-04, NFR-05 | measurement on the reference machine | gap, port phase |

Acceptance criteria for the port: the gate passes with the Rust binary in place of the script, and a differential harness runs both implementations over the same fixture set, comparing exit code, stdout, stderr and every produced file by hash, on Windows and on Linux.

## 10. Phases and gates

| Phase | State |
|---|---|
| Define | closed 2026-11-15: problem, goals, non-goals, requirements |
| Design | under review: architecture, interfaces, states, security, validation map |
| Build, Python 1.0.0 | shipped and gated: 15 of 15 |
| Build, port increments | not started |
| Release | not started |

Port increments, each closing at Gate 2:

1. P1: CLI parity and PNG parity, no page. Gate: argv matrix and decode parity.
2. P2: viewer page parity. Gate: byte comparison of the page against the Python output.
3. P3: Windows and Linux artefacts, subsystem choice, install path in the skill. Gate: no-console spawn test and redirected-handle capture test.
4. P4: differential harness in the gate. Gate: full fixture sweep green on both platforms.

Gate 1 waits for approval of this sheet and an answer to OQ-1.

## 11. Open questions

| Id | Question | Owner | Consequence if open |
|---|---|---|---|
| OQ-1 | A Rust binary is platform-specific, and the skill install is a two-file copy. Does the skill ship one binary per platform, build from source on install, or keep Python as the shipped path and treat the Rust build as an alternative? | maintainer | blocks the port's Release gate, not the Python audit |
| OQ-2 | After the port, does the version live in `Cargo.toml` alone, with SKILL.md carrying no second number? | maintainer | two version homes drift |
| OQ-3 | Windows stdout from Python is encoded in the ANSI codepage; Rust writes UTF-8 with LF. Replicate, or document the difference and compare message streams only? | maintainer | affects the differential harness |
| OQ-4 | The gate decodes images with zxing-cpp. Keep the gate in Python for the Rust line, or find a Rust decoder crate? | maintainer | gate portability |

## 12. Decision history

| Date | Decision | Reason |
|---|---|---|
| 2026-11-15 | Spec sheet written from the code, README and SKILL.md; status under review | the port needs a fixed behaviour target |
| 2026-11-15 | Error correction fixed at M, not selectable | one behaviour to test, one capacity limit to document |
| 2026-11-15 | `-f` decodes UTF-8 explicitly | platform default encoding must not change the payload |
| 2026-11-15 | The page carries no payload and no script | a photographed screen must not expose text, and a static file needs no script |
| 2026-11-15 | Header check on write, not a full decode | keeps the generator free of Pillow and zxing-cpp |
