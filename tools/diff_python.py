"""Differential sweep: the Python line against a built binary.

Runs both implementations over spec/corpus.txt and a fixed page set, then
compares exit code, printed path names, decoded payload, PNG dimensions and
page bytes. PNG bytes are not compared: two encoders write different valid
files for the same symbol.

Usage:
  python tools/diff_python.py --target rust-src/target/release/qr.exe
  python tools/diff_python.py --target bin/qr --python python/make-qr.py

The Python reference is removed at 0.8.0. This tool is the acceptance step
before that, not a permanent CI job; needs Pillow, zxing-cpp and segno.
"""
import argparse
import os
import shutil
import subprocess
import sys

from PIL import Image
import zxingcpp

TOOLS = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(TOOLS)
WORK = os.path.join(TOOLS, ".artifacts", "diff")
PY_SCRIPT = os.path.join(REPO, "python", "make-qr.py")
CORPUS = os.path.join(REPO, "spec", "corpus.txt")
EXTRA_PAYLOADS = [
    "ab",
    "https://example.com/qr-test",
    "caf\u00e9 cr\u00e8me",
    "line one\r\nline two",
    "\u3000wide\u3000",
]
PAGES = [
    ("qr code", "qr.png"),
    ("a b", "a b.png"),
    ('<b>&"x"</b>', "titled.png"),
    ("{png} {title} & <x>", "a&b.png"),
    ("caf\u00e9", "caf\u00e9.png"),
    ("", "empty.png"),
    ("it's a \"test\"", "q'1.png"),
    ("emoji \U0001F600", "e.png"),
    ("tab\tand\nnewline", "t.png"),
]
fails = []


def default_target():
    name = "qr.exe" if os.name == "nt" else "qr"
    return os.path.join(REPO, "bin", name)


def run(command, args, cwd):
    proc = subprocess.run(command + args, cwd=cwd, capture_output=True, text=True)
    return proc.returncode, proc.stdout, proc.stderr


def fresh(name):
    path = os.path.join(WORK, name)
    shutil.rmtree(path, ignore_errors=True)
    os.makedirs(path)
    return path


def decode(path):
    result = zxingcpp.read_barcode(Image.open(path))
    return result.text if result else None


def report(name, ok, detail=""):
    if not ok:
        fails.append(f"{name}: {detail}")


def stderr_kind(text):
    return text.strip().split(":", 1)[0]


def compare_payload(python, target, index, payload):
    py_dir = fresh(f"p{index}-py")
    rs_dir = fresh(f"r{index}-rs")
    args = [payload, "-o", "out.png"]
    py_code, py_out, py_err = run(python, args, py_dir)
    rs_code, rs_out, rs_err = run(target, args, rs_dir)
    if py_code != rs_code:
        return report(f"payload {index}", False, f"rc {py_code} vs {rs_code} {rs_err[:60]}")
    if py_code != 0:
        return report(
            f"payload {index}",
            stderr_kind(py_err) == stderr_kind(rs_err),
            f"{py_err[:50]} vs {rs_err[:50]}",
        )
    if [os.path.basename(line) for line in py_out.splitlines()] != [
        os.path.basename(line) for line in rs_out.splitlines()
    ]:
        return report(f"payload {index}", False, "printed paths differ")
    py_png = os.path.join(py_dir, "out.png")
    rs_png = os.path.join(rs_dir, "out.png")
    if Image.open(py_png).size != Image.open(rs_png).size:
        return report(
            f"payload {index}",
            False,
            f"size {Image.open(py_png).size} vs {Image.open(rs_png).size}",
        )
    py_text = decode(py_png)
    rs_text = decode(rs_png)
    if py_text != rs_text:
        return report(f"payload {index}", False, "decoded text differs")
    if rs_text != payload:
        return report(f"payload {index}", False, "binary decode is not the payload")
    return None


def compare_page(python, target, index, title, name):
    py_dir = fresh(f"h{index}-py")
    rs_dir = fresh(f"h{index}-rs")
    args = ["x", "-o", name, "--html", "--title", title]
    py_code, py_out, py_err = run(python, args, py_dir)
    rs_code, rs_out, rs_err = run(target, args, rs_dir)
    if py_code != 0 or rs_code != 0:
        return report(f"page {index}", False, f"rc {py_code} vs {rs_code} {rs_err[:60]}")
    page = os.path.splitext(name)[0] + ".html"
    with open(os.path.join(py_dir, page), "rb") as fh:
        py_bytes = fh.read()
    with open(os.path.join(rs_dir, page), "rb") as fh:
        rs_bytes = fh.read()
    return report(f"page {index}", py_bytes == rs_bytes, "page bytes differ")


def main(argv):
    parser = argparse.ArgumentParser(description="Compare the Python line with a built binary.")
    parser.add_argument("--python", default=PY_SCRIPT, help="reference script")
    parser.add_argument("--target", default=default_target(), help="binary to check")
    parser.add_argument("--corpus", default=CORPUS, help="hex payload corpus")
    args = parser.parse_args(argv)

    python = [sys.executable, os.path.abspath(args.python)]
    target = [os.path.abspath(args.target)]
    if not os.path.exists(args.python):
        print(f"DIFF missing reference: {args.python}")
        return 2
    if not os.path.exists(args.target):
        print(f"DIFF missing target: {args.target}")
        return 2

    shutil.rmtree(WORK, ignore_errors=True)
    os.makedirs(WORK)
    with open(args.corpus, encoding="ascii") as fh:
        rows = [line.strip() for line in fh if line.strip()]
    payloads = [bytes.fromhex(row).decode("utf-8") for row in rows] + EXTRA_PAYLOADS
    for index, payload in enumerate(payloads):
        compare_payload(python, target, index, payload)
    for index, (title, name) in enumerate(PAGES):
        compare_page(python, target, index, title, name)

    for failure in fails[:20]:
        print(f"FAIL {failure}")
    if len(fails) > 20:
        print(f"FAIL ... {len(fails) - 20} more")
    print(f"DIFF payloads={len(payloads)} pages={len(PAGES)} failures={len(fails)}")
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
