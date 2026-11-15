# Audit: qr skill, code and documents

Scope: `python/make-qr.py` (shipped generator), `tools/gate.py` (verification harness), and the text that makes claims about them (`SKILL.md`, `README.md`, `CHANGELOG.md`, `spec/qr-spec.md`).

Out of scope: the process manuals in `docs/`, the seed folder, git history, and the Rust port, which has no code yet.

Method: Part 1 was written from the code and from black-box runs of the shipped script with the decoder used by the gate (zxing-cpp), plus an import of `make-qr.py` to call `png_size` directly. Part 2 compares the documents against the code. Every finding names a file and a line. Test artefacts were produced in a scratch folder that is not part of the tree.

## Part 1: code findings

| Id | Severity | Location | Observation | Why it matters |
|---|---|---|---|---|
| M-01 | high | `python/make-qr.py:76` | `segno.make(data, error="m")` lets segno raise the error level when the symbol has spare capacity. A 19-character URL came out at level Q, confirmed by the decoder. | QR-10, SKILL.md, README and the changelog all state that error correction is fixed at M. It is a minimum, not a fixed level, so the documented contract is false and any test that asserts M fails. |
| M-02 | medium | `python/make-qr.py:76` | Payloads of 2 and 4 characters produce Micro QR symbols (versions M3 and M2). | Micro QR is a separate symbol family with narrower scanner support. The common pure-Rust encoders (qrcode, fast_qr, qrcodegen) do not emit it, so a port either adds a bespoke encoder or changes the symbol family. |
| M-03 | medium | `python/make-qr.py:62-63` | `open(..., "r", encoding="utf-8")` translates line endings; a file holding `a\r\nb\r\n` decodes back as `a\nb`. | The payload is silently rewritten on read. A file whose bytes matter (multi-line Wi-Fi string, structured token) is not encoded as stored, and no document mentions it. |
| M-04 | medium | `python/make-qr.py:61-63` | Read failures are uncaught. A missing file exits 1 with `FileNotFoundError` and a traceback; a non-UTF-8 file exits 1 with `UnicodeDecodeError` and a traceback. | QR-19 requires one line on stderr, exit 1 and no traceback. The traceback also prints absolute paths of the script and the caller. |
| M-05 | medium | `python/make-qr.py:74`, `75-77`, `85`, `92-94` | Write failures are uncaught: a parent path that is a file fails in `os.makedirs` at line 74 with a traceback, and a blocked page path gives a traceback after the PNG path has already gone to stdout. `os.remove` at line 85 is unguarded too. | QR-19 and QR-14. A caller that reads stdout sees a success path while the run has failed, which is the exact case G3 exists to prevent. |
| M-06 | low | `python/make-qr.py:59` | `if args.file and args.text` treats an empty string as "no text", so `"" -f file` encodes the file at exit 0. | QR-03 says the two sources are mutually exclusive. A caller that means to reject an empty payload instead gets a file-backed one. |
| M-07 | low | `python/make-qr.py:36-41`, `82-88` | The post-write check reads only the 24-byte header. A file truncated to 100 bytes still reports the expected dimensions. | The check proves the module count, not that the image is complete. Only a decode proves completeness, and the generator deliberately does not carry a decoder. |
| M-08 | low | `python/make-qr.py:15-31` | The viewer page has no viewport meta tag. | A phone lays the page out at a 980 px default layout viewport, so `70vmin` resolves against 980 px, not the screen. The default is browser behaviour from the specification; the missing tag is measured. The page is sized for scanning in intent but not in fact. |
| M-09 | low | `python/make-qr.py:91-93` | With `-o` and no file extension, the page references an image name without an extension. | A file:// browser has no extension for MIME guessing and may refuse to render it. Not observed in a browser; resolved by a doc line, not a code change. |
| M-10 | low | `python/make-qr.py:55-56`, `76-77` | No raster guard. 2331 bytes at `--scale 100` is 18500 px per side, about 342 Mpx. | The accepted range allows an output that no scanner needs and that costs hundreds of MB of raster. Not run at full size on purpose. |
| M-11 | trivial | `python/make-qr.py:13` | `DataOverflowError` is imported from `segno.encoder`, a private module, although `segno.DataOverflowError` is public and the same object. | A segno refactor can break the import at module load, which would break every run, not just an overflow. |
| M-12 | trivial | `python/make-qr.py:36-41`, `86` | `png_size` returns `None` for a file that is not a PNG, and the message then prints `None`. | Acceptable as is; recorded so it is not raised again. |

| Id | Severity | Location | Observation | Why it matters |
|---|---|---|---|---|
| G-01 | medium | `tools/gate.py:37` | The test URL is `https://asusvivos14.forest-piranha.ts.net/ntfy-proj?x=1`, a private tailnet host. | It is not a credential, but it names a private endpoint and travels with the repo. A public example host serves the same purpose. |
| G-02 | low | `tools/gate.py` | 15 checks cover 11 of the 22 functional requirements. No check for QR-04, QR-07, QR-10, QR-13, QR-16 to QR-22. | The uncovered paths are the ones that failed in this audit: the error level, the conflict rule and the error handling. |
| G-03 | low | `tools/gate.py:39-76` | No assertion on error level, symbol family or the viewer markup beyond the payload text. | M-01, M-02 and M-08 survived because nothing looked. |
| G-04 | trivial | `tools/gate.py:12`, `15`, `36`, `55` | `import shutil` sits after executable statements, `os.makedirs(WORK)` runs twice, and `"tk_" not in page` is redundant once `secret not in page` holds. | Tidy only; no behaviour. |
| G-05 | low | `tools/gate.py:23-27` | A missing Pillow or zxing-cpp fails with a traceback at the first decode. | The gate is the one tool a new machine runs first; it should name the missing package and stop. |

## Part 2: document cross-check

| Id | Severity | Location | Claim | Code |
|---|---|---|---|---|
| D-01 | high | `SKILL.md:42`, `README.md:49`, `CHANGELOG.md:9`, `spec/qr-spec.md` QR-10 | "Error correction is fixed at M." | False; the level is raised when capacity allows (M-01). |
| D-02 | medium | `SKILL.md:24`, `spec/qr-spec.md` QR-02 and QR-12 | `-f` reads UTF-8 and strips surrounding whitespace; the PNG decodes back to the payload. | It also rewrites CRLF and lone CR, so "the payload" is neither the file bytes nor the stripped text (M-03). |
| D-03 | low | `README.md:64` | "No open defects" (line dated 2026-11-13). | Contradicted by this audit. |
| D-04 | low | `SKILL.md:58` | Keep the quiet zone at 4 modules or more. | `--border` accepts 0. Advice, not a contract; recorded to close it. |
| D-05 | - | `seed/README.md:19-20`, `seed/skill- doc-needs-editing.md` | Examples reference `phone-token.png` and `phone-token.html`. | Those files lived on the Desktop and were never moved. History, not product. |
| D-06 | - | `docs/python-to-rust-refactor-pitfalls.md:126, 145, 156` | "The fix used in this port", "used here: 28 cases", "Measured on the shipped exe". | Examples from an earlier port. A general manual; not a claim about this repo. |
| D-07 | - | `README.md:13`, `spec/qr-spec.md` section 9 | "15-check". | True: 15 `check()` calls. |
| D-08 | - | `spec/qr-spec.md` QR-22 | `--version` prints the version. | Not implemented; QR-22 is a 0.8.0 requirement. Not a defect. |

## Evidence log

Probes ran against the shipped script, working directory `audit-scratch`, decoder zxing-cpp 3.1.1 with Pillow 12.3.0.

- `-f nope.txt` -> exit 1, `FileNotFoundError: [Errno 2] No such file or directory: 'nope.txt'`, no PNG written.
- `-f bad.bin` (bytes `ff fe 00 01`) -> exit 1, `UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff`.
- `"" -f e.txt -o emptyboth.png` -> exit 0, one PNG path, decodes to `fromfile`.
- `-f crlf.txt` (`a\r\nb\r\n`) -> exit 0, decodes to `a\nb`.
- `-o adir` (a directory) -> exit 1, `FAIL store: Unknown file extension`; clean, but the message names an extension problem for a path problem.
- `-o afile/sub/x.png` (parent is a file) -> exit 1, `FileNotFoundError: [WinError 3]`.
- `-o pagefail.png --html` with `pagefail.html` a directory -> stdout carries the PNG path, exit 1, `PermissionError` traceback.
- Payloads 2, 4, 19, 100, 500, 2331 bytes: levels from the decoder M, M, Q, M, M, M; versions M3, M2, 2, 6, 17, 40. With `boost_error=False` every level is M.
- `png_size` on a 330 x 330 PNG and on the same file truncated to 100 bytes: both `(330, 330)`.
- Empty stdin with no source -> exit 2, `provide text or --file`; stdin is not read.
- Generated page contains no `name="viewport"`.
- 2331 bytes at level M is version 40, 177 modules; at `--scale 100` that is 18500 px per side.
- Gate run with a shadowed `zxingcpp` module that raises `ImportError`: exit 1 after two PASS lines, with a traceback.

## Counts

25 rows: high 2, medium 6, low 10, trivial 3, out of scope 2, non-findings 2.

## Re-audit

A second pass checked every row against the code and re-ran the probes. Corrections, all applied above:

- The decoder is zxing-cpp 3.1.1, not the 2.x the first draft of the evidence log claimed.
- D-03 sits at `README.md:64`, not 60.
- M-05 also fails at line 74, inside `os.makedirs`, before the try block.
- M-02 no longer claims that no Rust encoder emits Micro QR. The common pure-Rust crates do not; `rxing` is a ZXing port and does. The finding stands on scanner support and on parity cost.
- M-08 marks the 980 px layout viewport as browser specification rather than a measurement.
- M-09 marks the browser refusal as unverified.
- G-05 now has a measured reproduction instead of an inspection.
- The counts line now totals 25 and separates non-findings from out-of-scope items.

No finding was withdrawn. The severity order is unchanged.
