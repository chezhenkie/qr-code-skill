"""Black-box acceptance gate for the shipped generator.

Runs bin\qr.exe on Windows and bin/qr elsewhere, or the binary given with
--target. Needs zxing-cpp and Pillow to decode the produced PNGs.
"""
import os
import re
import shutil
import struct
import subprocess
import sys

TOOLS = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(TOOLS)
WORK = os.path.join(TOOLS, ".artifacts")
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def default_target():
    return os.path.join(REPO, "bin", "qr.exe" if os.name == "nt" else "qr")


TARGET = (
    os.path.abspath(sys.argv[sys.argv.index("--target") + 1])
    if "--target" in sys.argv
    else default_target()
)
fails = []

try:
    import zxingcpp
    from PIL import Image
except ImportError as exc:
    print(f"GATE missing dependency: {exc.name}, install zxing-cpp and pillow")
    sys.exit(2)

shutil.rmtree(WORK, ignore_errors=True)
os.makedirs(WORK, exist_ok=True)


def art(*parts):
    return os.path.join(WORK, *parts)


def run(args, stdin=None):
    p = subprocess.run([TARGET] + args, cwd=WORK, capture_output=True, text=True, stdin=stdin)
    return p.returncode, p.stdout.strip(), p.stderr.strip()


def png_probe(path):
    """Return (IHDR size or None, whether the file ends with an IEND chunk)."""
    with open(path, "rb") as handle:
        head = handle.read(24)
        end = os.fstat(handle.fileno()).st_size
        tail = b""
        if end >= 12:
            handle.seek(end - 12)
            tail = handle.read(12)
    complete = len(tail) == 12 and tail[4:8] == b"IEND"
    if len(head) < 24 or head[:8] != PNG_SIGNATURE or head[12:16] != b"IHDR":
        return None, complete
    return struct.unpack(">II", head[16:24]), complete


def barcode(path):
    return zxingcpp.read_barcode(Image.open(path))


def decode(path):
    r = barcode(path)
    return r.text if r else None


def payload(path):
    r = barcode(path)
    return r.bytes if r else None


def ec_level(path):
    r = barcode(path)
    return r.ec_level if r else None


def check(name, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + name + ("" if cond else " :: " + detail))
    if not cond:
        fails.append(name)


url = "https://example.com/qr-test"

code, out, err = run([url, "-o", "plain.png"])
lines = out.splitlines()
check("text exit 0", code == 0, err)
check("prints one absolute path", len(lines) == 1 and os.path.isabs(lines[0]), out)
check("decode matches", decode(art("plain.png")) == url, str(decode(art("plain.png"))))

secret = "tk_0123456789abcdef0123456789abcdef"
with open(art("secret.txt"), "w", encoding="utf-8") as fh:
    fh.write("  " + secret + "\n")
code, out, err = run(["-f", "secret.txt", "-o", os.path.join("deep", "secret.png"), "--scale", "12", "--html", "--title", "phone token"])
paths = out.splitlines()
check("file exit 0", code == 0, err)
check("two paths png then html", len(paths) == 2 and paths[0].endswith(".png") and paths[1].endswith(".html"), out)
check("strips file whitespace", decode(art("deep", "secret.png")) == secret, str(decode(art("deep", "secret.png"))))
page = open(art("deep", "secret.html"), encoding="utf-8").read()
check("viewer references png", 'src="secret.png"' in page, page[:80])
check("viewer hides text", secret not in page, "text leaked")

img = Image.open(art("deep", "secret.png"))
check("scale 12 pixels", img.width == img.height and img.width % 12 == 0, f"{img.width}x{img.height}")
img = Image.open(art("plain.png"))
check("default scale 10 pixels", img.width == img.height and img.width % 10 == 0, f"{img.width}x{img.height}")

code, out, err = run(["x", "-f", "secret.txt", "-o", "both.png"])
check("rejects text plus file", code == 2 and "not both" in err, f"{code} {err[-60:]}")

code, out, err = run([""])
check("rejects empty text", code == 2 and "empty input" in err, f"{code} {err[-60:]}")

code, out, err = run([])
check("rejects no source", code == 2 and "provide text or --file" in err, f"{code} {err[-60:]}")

code, out, err = run(["x" * 3000, "-o", "huge.png"])
check("rejects oversize", code == 1 and err.startswith("FAIL store") and not os.path.exists(art("huge.png")), f"{code} {err[:80]}")

code, out, err = run(["", "-f", "secret.txt", "-o", "emptyboth.png"])
check("rejects empty text with file", code == 2 and "not both" in err, f"{code} {err[-60:]}")

short = "ab"
code, out, err = run([short, "-o", "short.png"])
check("short payload regular 21 modules", code == 0 and Image.open(art("short.png")).width == 290 and decode(art("short.png")) == short, f"{code} {err}")
check("short payload level M", ec_level(art("short.png")) == "M", str(ec_level(art("short.png"))))

url19 = "https://example.com"
code, out, err = run([url19, "-o", "url19.png"])
check("url level M not boosted", code == 0 and ec_level(art("url19.png")) == "M", str(ec_level(art("url19.png"))))

check("viewport tag present", 'name="viewport"' in page, page[:120])

with open(art("crlf.txt"), "wb") as fh:
    fh.write(b"a\r\nb\r\n")
code, out, err = run(["-f", "crlf.txt", "-o", "crlf.png"])
check("crlf payload preserved", code == 0 and decode(art("crlf.png")) == "a\r\nb", f"{code} {err}")

with open(art("strip.txt"), "wb") as fh:
    fh.write(b"\x1ca\x1f")
code, out, err = run(["-f", "strip.txt", "-o", "strip.png"])
with open(art("inner.txt"), "wb") as fh:
    fh.write(b"a\x1cb")
inner_code, inner_out, inner_err = run(["-f", "inner.txt", "-o", "inner.png"])
check("strip set boundary", code == 0 and inner_code == 0 and payload(art("strip.png")) == b"a" and payload(art("inner.png")) == b"a\x1cb", f"{code} {inner_code} {err} {inner_err}")

title = '<b>&"x"</b>'
code, out, err = run([secret, "-o", "titled.png", "--html", "--title", title])
titled = open(art("titled.html"), encoding="utf-8").read()
check("title escaped in page", code == 0 and "&lt;b&gt;&amp;&quot;x&quot;&lt;/b&gt;" in titled, titled[:120])
check("title never encoded", decode(art("titled.png")) == secret, str(decode(art("titled.png"))))

code, out, err = run(["first", "-o", "over.png"])
code, out, err = run(["second", "-o", "over.png", "--scale", "12"])
check("overwrite replaces the file", code == 0 and decode(art("over.png")) == "second", f"{code} {err}")

code, out, err = run(["x", "-o", os.path.join("n1", "n2", "n3", "qr.png")])
check("creates nested parents", code == 0 and os.path.exists(art("n1", "n2", "n3", "qr.png")), f"{code} {err}")

code, out, err = run(["-h"])
check("help exits 0", code == 0 and "usage" in out.lower() and err == "", f"{code} {err[:80]}")

with open(art("secret.txt"), encoding="utf-8") as fh:
    code, out, err = run(["stdin-check", "-o", "stdin.png"], stdin=fh)
check("stdin is ignored", code == 0 and decode(art("stdin.png")) == "stdin-check", str(decode(art("stdin.png"))))

code, out, err = run(["-f", "missing.txt", "-o", "missing.png"])
check("missing file is one FAIL read line", code == 1 and out == "" and err.startswith("FAIL read") and "Traceback" not in err and not os.path.exists(art("missing.png")), f"{code} {err[:90]}")

with open(art("latin1.bin"), "wb") as fh:
    fh.write(b"\xff\xfe\x00bad")
code, out, err = run(["-f", "latin1.bin", "-o", "latin1.png"])
check("non utf8 file is one FAIL read line", code == 1 and out == "" and err.startswith("FAIL read") and "Traceback" not in err, f"{code} {err[:90]}")

with open(art("blocker"), "w", encoding="utf-8") as fh:
    fh.write("x")
code, out, err = run(["x", "-o", os.path.join("blocker", "hidden.png")])
check("blocks output parent with FAIL write", code == 1 and out == "" and err.startswith("FAIL write") and "Traceback" not in err, f"{code} {err[:90]}")

os.makedirs(art("blocked.html"), exist_ok=True)
code, out, err = run(["x", "-o", "blocked.png", "--html"])
check("blocks page with FAIL write", code == 1 and out == "" and err.startswith("FAIL write") and "Traceback" not in err, f"{code} {err[:90]}")
check("page failure keeps the PNG", os.path.exists(art("blocked.png")), "png removed")

trunc = art("trunc.png")
with open(art("plain.png"), "rb") as src, open(trunc, "wb") as dst:
    dst.write(src.read(100))
size, complete = png_probe(trunc)
full_size, full_complete = png_probe(art("plain.png"))
check("png tail detects truncation", size is not None and not complete and full_complete, f"{size} {complete} {full_complete}")

code, out, err = run(["--version"])
check(
    "version prints one line",
    code == 0 and len(out.splitlines()) == 1 and re.fullmatch(r"\d+\.\d+\.\d+", out) is not None and err == "",
    f"{code} {out!r} {err[:40]}",
)

code, out, err = run(["x", "--scale", "0", "-o", "bad.png"])
check("rejects scale 0", code == 2 and "--scale must be 1-100" in err, f"{code} {err[-60:]}")
code, out, err = run(["x", "--border", "17", "-o", "bad2.png"])
check("rejects border 17", code == 2 and "--border must be 0-16" in err, f"{code} {err[-60:]}")

print(f"GATE failures={len(fails)}")
sys.exit(1 if fails else 0)
