"""Compare segno's symbol choice with the Rust encoder over a fixed corpus.

The corpus is committed as hex lines in spec/corpus.txt so both sides read the
same bytes. The Python table is the reference; the Rust table comes from
`cargo run --example versions < spec/corpus.txt`.

Payloads where segno selects kanji mode are excluded: the port has no kanji
segment and writes UTF-8 byte mode instead, so it can need more versions.

Usage:
  python tools/parity.py --freeze     rewrite the corpus
  python tools/parity.py --table      print "version<TAB>modules<TAB>hex" per payload
  python tools/parity.py --check FILE compare a Rust table with the segno table
"""
import os
import sys

import segno

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(REPO, "spec", "corpus.txt")


def build_corpus():
    cases = []

    def add(text):
        if text and text not in cases:
            cases.append(text)

    for n in (1, 2, 5, 10, 20, 40, 80, 160, 400, 1000):
        add("9" * n)
        add("A" * n)
        add("a" * n)
        add(("AB12" * n)[:n])
        add(("https://example.com/" + "x" * n)[:n])
        if n >= 3:
            add("a" + "1" * (n - 2) + "b")
            add("a" + "A" * (n - 2) + "b")
            add("1" + "a" * (n - 2) + "1")
    add("https://example.com/qr-test")
    add("tk_0123456789abcdef0123456789abcdef")
    add("m\u00fcnchen")
    add("\u6f22\u5b57\u30c6\u30b9\u30c8")
    add("emoji \U0001F600")
    add("a b")
    add("a-b_c")
    add("x:y/z")
    add("$%*+-./:")
    add("HELLO WORLD")
    add("hello world 12345678901234567890")
    add("MiXeD CaSe 42")
    add("!" * 200)
    add(" " * 50)

    out = []
    for text in cases:
        try:
            code = segno.make(text, error="m", micro=False, boost_error=False)
        except Exception:
            continue
        if code.mode == "kanji":
            continue
        out.append(text)
    return out


def symbol(text):
    code = segno.make(text, error="m", micro=False, boost_error=False)
    return code.version, code.symbol_size(scale=1, border=0)[0]


def freeze():
    corpus = build_corpus()
    with open(CORPUS, "w", encoding="ascii", newline="\n") as fh:
        for text in corpus:
            fh.write(text.encode("utf-8").hex() + "\n")
    print(f"frozen {len(corpus)} payloads to {os.path.relpath(CORPUS, REPO)}")


def read_corpus():
    with open(CORPUS, encoding="ascii") as fh:
        return [bytes.fromhex(line.strip()).decode("utf-8") for line in fh if line.strip()]


def table():
    rows = []
    for text in read_corpus():
        version, modules = symbol(text)
        rows.append(f"{version}\t{modules}\t{text.encode('utf-8').hex()}")
    return rows


def check(path):
    want = table()
    with open(path, encoding="utf-8") as fh:
        got = [line.rstrip("\n") for line in fh if line.strip()]
    bad = 0
    if len(got) != len(want):
        print(f"row count: want {len(want)} got {len(got)}")
        bad += 1
    for index, (a, b) in enumerate(zip(want, got)):
        if a != b:
            bad += 1
            if bad <= 10:
                av, am, ah = a.split("\t")
                parts = b.split("\t")
                print(f"  row {index}: want v{av} {am}px, got {' '.join(parts[:2])}")
    print(f"PARITY rows={len(want)} mismatches={bad}")
    return 1 if bad else 0


def main(argv):
    if "--freeze" in argv:
        freeze()
        return 0
    if "--table" in argv:
        for row in table():
            print(row)
        return 0
    if "--check" in argv:
        return check(argv[argv.index("--check") + 1])
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
