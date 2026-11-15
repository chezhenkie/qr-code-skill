"""Freeze and check the command-line surface of the generator.

The fixture in spec/argv-matrix.json records exit code, stdout and stderr for
each case. Cases run against bin\\qr.exe on Windows and bin/qr elsewhere; the
harness normalises the program name and scratch paths, collapses whitespace in
stderr, and compares help stdout with whitespace collapsed because the usage
line wraps on the program name.

Usage:
  python tools/argv_matrix.py                         check the shipped binary
  python tools/argv_matrix.py --target PATH           check another binary
  python tools/argv_matrix.py --freeze                rewrite the fixture from the shipped binary
  python tools/argv_matrix.py --freeze --target PATH  rewrite it from another binary
"""
import json
import os
import re
import shutil
import subprocess
import sys

TOOLS = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(TOOLS)
FIXTURE = os.path.join(REPO, "spec", "argv-matrix.json")
WORK = os.path.join(TOOLS, ".artifacts", "matrix")


def default_target():
    return os.path.join(REPO, "bin", "qr.exe" if os.name == "nt" else "qr")

CASES = [
    {"id": "no-args", "argv": []},
    {"id": "text-default", "argv": ["x"]},
    {"id": "text-out", "argv": ["x", "-o", "a.png"]},
    {"id": "out-equals", "argv": ["x", "--out=a.png"]},
    {"id": "scale-equals", "argv": ["x", "--scale=12"]},
    {"id": "prefix-scale", "argv": ["x", "--sc", "12"]},
    {"id": "ambiguous-h", "argv": ["x", "--h"]},
    {"id": "unknown-long", "argv": ["x", "--bogus"]},
    {"id": "unknown-cluster", "argv": ["-xh"]},
    {"id": "help", "argv": ["-h"], "stdout_flat": True},
    {"id": "help-cluster", "argv": ["-hx"], "stdout_flat": True},
    {"id": "help-long", "argv": ["--help"], "stdout_flat": True},
    {"id": "missing-value-scale", "argv": ["x", "--scale"]},
    {"id": "bad-int", "argv": ["x", "--scale", "abc"]},
    {"id": "negative-border", "argv": ["x", "--border", "-1"]},
    {"id": "range-scale", "argv": ["x", "--scale", "0"]},
    {"id": "range-border", "argv": ["x", "--border", "17"]},
    {"id": "out-looks-like-option", "argv": ["x", "--out", "--html"]},
    {"id": "double-dash", "argv": ["--", "--scale"]},
    {"id": "dash-text", "argv": ["-"]},
    {"id": "empty-text", "argv": [""]},
    {"id": "two-positionals", "argv": ["x", "y"]},
    {"id": "dup-out", "argv": ["x", "-o", "a.png", "-o", "b.png"]},
    {"id": "file-ok", "argv": ["-f", "secret.txt", "-o", "s.png"], "setup": {"secret.txt": "  tok \n"}},
    {"id": "file-missing", "argv": ["-f", "nope.txt"], "stderr_prefix": "FAIL read:"},
    {"id": "nested-out", "argv": ["x", "-o", "a/b/c.png"]},
    {"id": "html-title", "argv": ["x", "--html", "--title", "a b"]},
]


def normalize(text, work, prefix):
    for root in {work, work.replace(os.sep, "/")}:
        text = text.replace(root, "<dir>")
    text = text.replace(os.sep, "/")
    for arg in prefix:
        name = os.path.basename(arg)
        if not name:
            continue
        text = text.replace(f"usage: {name}", "usage: PROG")
        text = text.replace(f"{name}: error:", "PROG: error:")
    return text


def flatten(text):
    return re.sub(r"\s+", " ", text).strip()


def run_case(case, prefix):
    work = os.path.join(WORK, case["id"])
    shutil.rmtree(work, ignore_errors=True)
    os.makedirs(work)
    for name, content in case.get("setup", {}).items():
        with open(os.path.join(work, name), "w", encoding="utf-8", newline="") as fh:
            fh.write(content)
    p = subprocess.run(prefix + case["argv"], cwd=work, capture_output=True, text=True)
    return {
        "rc": p.returncode,
        "stdout": normalize(p.stdout, work, prefix),
        "stderr": flatten(normalize(p.stderr, work, prefix)),
    }


def verify(prefix, expect, quiet=False):
    bad = []
    for case in CASES:
        got = run_case(case, prefix)
        want = expect[case["id"]]
        if case.get("stdout_flat"):
            stdout_ok = flatten(got["stdout"]) == flatten(want["stdout"])
        else:
            stdout_ok = got["stdout"] == want["stdout"]
        if case.get("stderr_prefix"):
            stderr_ok = got["stderr"].startswith(case["stderr_prefix"])
            want_err = case["stderr_prefix"] + " ..."
        else:
            stderr_ok = got["stderr"] == want["stderr"]
            want_err = want["stderr"]
        ok = got["rc"] == want["rc"] and stdout_ok and stderr_ok
        if not quiet:
            print(("PASS " if ok else "FAIL ") + case["id"])
        if not ok:
            bad.append((case["id"], want, got, want_err))
    return bad


def freeze(prefix):
    version = subprocess.run(prefix + ["--version"], capture_output=True, text=True)
    data = {
        "reference": {
            "program": version.stdout.strip() if version.returncode == 0 else "unknown",
            "platform": sys.platform,
        },
        "cases": {case["id"]: run_case(case, prefix) for case in CASES},
    }
    with open(FIXTURE, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(data, fh, indent=2, sort_keys=True)
        fh.write("\n")
    print(f"frozen {len(CASES)} cases to {os.path.relpath(FIXTURE, REPO)}")


def main(argv):
    target = default_target()
    if "--target" in argv:
        target = os.path.abspath(argv[argv.index("--target") + 1])
    prefix = [os.path.abspath(target)]
    if "--freeze" in argv:
        freeze(prefix)
        return 0
    with open(FIXTURE, encoding="utf-8") as fh:
        expect = json.load(fh)["cases"]
    bad = verify(prefix, expect)
    for case_id, want, got, want_err in bad:
        print(f"  {case_id}: want rc={want['rc']} stdout={want['stdout'][:90]!r} stderr={want_err[:90]!r}")
        print(f"  {case_id}: got  rc={got['rc']} stdout={got['stdout'][:90]!r} stderr={got['stderr'][:90]!r}")
    print(f"MATRIX cases={len(CASES)} failures={len(bad)}")
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
