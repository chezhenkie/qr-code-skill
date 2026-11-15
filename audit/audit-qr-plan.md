# Audit plan: qr skill 0.7.1

Triaged from `audit/audit-qr-code.md`. Accepted items only. One item per change, each with the files it touches, the test that fails when the change is reverted, and the risk of doing it.

Scope of the release: fixes and tests on the Python line. No feature work. Version is a patch bump to 0.7.1.

## Accepted

| Id | Change | Files | Test that proves it | Risk |
|---|---|---|---|---|
| P-01 | Pass `boost_error=False` and `micro=False` so every payload is a regular QR symbol at level M. | `python/make-qr.py:76` | Gate decodes a short payload and a URL and asserts level `M` and a regular symbol: a 2-character payload must be 21 modules, 290 px at scale 10. Today the URL reports `Q` and two characters give an M3 Micro symbol. | Small payloads get a slightly larger symbol than before and lose the extra margin they had by accident. Output bytes change; no test compares bytes. |
| P-02 | Read `-f` with `newline=""` so line endings are preserved, and map `OSError` and `UnicodeDecodeError` to one `FAIL read:` line with exit 1. | `python/make-qr.py:61-63` | CRLF fixture decodes back with `\r\n`; missing file and non-UTF-8 file exit 1 with a `FAIL read:` line and no `Traceback`. | Windows files that hold CRLF now encode `\r\n`. Documented in SKILL.md. |
| P-03 | Hold both path strings, write the page before printing, and map `OSError` on save and page write to one `FAIL write:` line with exit 1. Guard the `os.remove` in the mismatch branch. | `python/make-qr.py:75-94` | A parent path that is a file exits 1 with `FAIL write:` and empty stdout; a page path blocked by a directory exits 1 with empty stdout. Today stdout carries the PNG path. | Print order changes. Nothing else does. |
| P-04 | Treat an empty text argument as a source for the conflict rule: `args.text is not None`. | `python/make-qr.py:59` | `"" -f file` exits 2 with `not both`; today it exits 0 and encodes the file. | None. |
| P-05 | After the size check, require the `IEND` chunk in the last 12 bytes; remove the file and fail if it is absent. | `python/make-qr.py:82-88` | The gate imports the module and runs the check on a file truncated at 100 bytes; today it passes. | None. The check reads 12 bytes. |
| P-06 | Add `<meta name="viewport" content="width=device-width, initial-scale=1">` to the viewer page. | `python/make-qr.py:15-31` | Gate asserts the tag is present; a phone check by the operator confirms the size. | None. |
| P-07 | Extend the gate to the 11 uncovered requirements: no source, nested parent creation, level M and symbol family, truncation, title escaping and title-never-encoded, overwrite, missing and non-UTF-8 input, blocked output, help exit 0, stdin ignored, CRLF fidelity. | `tools/gate.py` | The gate itself. Target 28 or more checks, all green. | A wrong check costs a false failure. Each one carries the reproduction from the audit. |
| P-08 | Replace the private tailnet URL with `https://example.com/qr-test`. | `tools/gate.py:37` | Gate still decodes the payload exactly. | None. |
| P-09 | Gate hygiene: single `os.makedirs`, `shutil` at the top, drop the redundant `"tk_"` condition, and name a missing decoder dependency then exit 2. | `tools/gate.py:12-36, 55` | Gate run; a decoder import failure prints one line instead of a traceback. | None. |
| P-10 | Align the text: fixed-M wording where it still says boosted, the regular-QR rule, the line-ending rule for `-f`, the viewport, the raster limit note, the `.png` extension note, and a README status line that no longer claims zero defects. Update the spec validation map and the decision history. | `SKILL.md`, `README.md`, `CHANGELOG.md`, `spec/qr-spec.md` | The doc sweep: paths resolve, counts match the gate, version fields agree. | None. |
| P-11 | Version 0.7.1 in its single home, a 0.7.1 changelog chapter: fixes only. Resync the install and record the hashes. | `SKILL.md`, `CHANGELOG.md`, install folder | Hashes match; `python tools\gate.py` green. | None. |

## Rejected

- R-01 `D-05` seed examples referencing Desktop files: history, not product; the folder is documented as historical.
- R-02 `D-06` pitfalls manual speaking from an earlier port: a general manual, not a claim about this repo.
- R-03 `D-04` quiet-zone advice against `--border 0`: advice, not a contract; the range is deliberate.
- R-04 `M-12` `None` in the size message: the only way to reach it is a failed write, which P-03 now reports properly.
- R-05 `M-09` extension-less output as a code change: one line in SKILL.md, no behaviour change.
- R-06 A raster cap as a new error: the documented range is a product choice; P-10 records the size instead.

## Open decisions

- O-01 closed: regular QR only. `micro=False` is part of P-01. Micro QR is out of scope for both lines, because scanner support varies and the common pure-Rust encoders do not emit it.
- O-02 Error level for the port: P-01 pins M and regular QR, so the Rust encoder has one mode to reproduce. Confirmed as the intent, no action now.

## Execution order

1. P-04, P-01, P-05, P-06: generator contract, each with its gate check.
2. P-02, P-03: error paths, each with its gate check.
3. P-07, P-08, P-09: gate coverage and hygiene.
4. P-10: text alignment.
5. P-11: version, changelog, install sync.
6. Gate green, install hashes recorded, push.

Commits: one per group above, each naming the findings it closes. The audit document stays unchanged as the record; this plan carries the outcome.

## Acceptance

- `python tools\gate.py` exits 0 with no FAIL line, and the check count matches the number quoted in README and spec.
- No error path prints a traceback: missing input, non-UTF-8 input, blocked output, blocked page, oversize payload.
- A generated symbol decodes at level M, and no payload produces Micro QR: two characters gives a 21-module regular symbol.
- The viewer page contains the viewport tag and no payload text.
- Install folder files hash-equal to the repo copies.
- The spec validation map lists no uncovered functional requirement except QR-22, which belongs to 0.8.0.
