"""Black-box acceptance gate for the shipped generator.

Runs the binary for the host architecture from agent-skill/bin, or the one given with
--target. Needs zxing-cpp and Pillow to decode the produced PNGs.
"""
import atexit
import os
import platform
import re
import shutil
import stat
import struct
import subprocess
import sys

TOOLS = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(TOOLS)
WORK = os.path.join(TOOLS, ".artifacts")
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def default_target():
    arm = platform.machine().lower() in ("arm64", "aarch64")
    if os.name == "nt":
        return os.path.join(REPO, "agent-skill", "bin", "qr-arm64.exe" if arm else "qr.exe")
    if sys.platform == "darwin":
        if not arm:
            raise SystemExit("no Intel macOS binary; pass --target or build from rust-src")
        return os.path.join(REPO, "agent-skill", "bin", "qr-macos-arm64")
    return os.path.join(REPO, "agent-skill", "bin", "qr-arm64" if arm else "qr")


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
atexit.register(shutil.rmtree, WORK, ignore_errors=True)


def art(*parts):
    return os.path.join(WORK, *parts)


def run(args, stdin=None, cwd=None):
    p = subprocess.run([TARGET] + args, cwd=cwd or WORK, capture_output=True, text=True, stdin=stdin)
    return p.returncode, p.stdout.strip(), p.stderr.strip()


def run_bytes(args, cwd):
    p = subprocess.run([TARGET] + args, cwd=cwd, capture_output=True, stdin=subprocess.DEVNULL)
    return p.returncode, p.stdout, p.stderr


TERM_ON = "\x1b[30;107m"
TERM_OFF = "\x1b[0m"
TERM_GLYPHS = {
    " ": (False, False),
    "\u2588": (True, True),
    "\u2580": (True, False),
    "\u2584": (False, True),
}


def term_image(text):
    """Rebuild the 1-bit grid from painted half-block lines, or None."""
    lines = text.splitlines()
    if not lines:
        return None
    cells = []
    for line in lines:
        if not (line.startswith(TERM_ON) and line.endswith(TERM_OFF)):
            return None
        body = line[len(TERM_ON) : -len(TERM_OFF)]
        if any(ch not in TERM_GLYPHS for ch in body):
            return None
        cells.append(body)
    width = max(len(body) for body in cells)
    image = Image.new("L", (width, len(cells) * 2), 255)
    pixels = image.load()
    for row, body in enumerate(cells):
        for x, ch in enumerate(body):
            upper, lower = TERM_GLYPHS[ch]
            if upper:
                pixels[x, row * 2] = 0
            if lower:
                pixels[x, row * 2 + 1] = 0
    return image


def term_decode(text):
    image = term_image(text)
    if image is None:
        return None, None
    wide = image.resize((image.width * 8, image.height * 8), Image.NEAREST)
    return image, zxingcpp.read_barcode(wide)


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
    with Image.open(path) as image:
        return zxingcpp.read_barcode(image)


def image_size(path):
    with Image.open(path) as image:
        return image.size


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
with open(art("deep", "secret.html"), encoding="utf-8") as fh:
    page = fh.read()
check("viewer references png", 'src="secret.png"' in page, page[:80])
check("viewer hides text", secret not in page, "text leaked")

w, h = image_size(art("deep", "secret.png"))
check("scale 12 pixels", w == h and w % 12 == 0, f"{w}x{h}")
w, h = image_size(art("plain.png"))
check("default scale 10 pixels", w == h and w % 10 == 0, f"{w}x{h}")

code, out, err = run(["x", "-f", "secret.txt", "-o", "both.png"])
check("rejects text plus file", code == 2 and "not both" in err, f"{code} {err[-60:]}")

code, out, err = run([""])
check("rejects empty text", code == 2 and "empty input" in err, f"{code} {err[-60:]}")

code, out, err = run([])
check("rejects no source", code == 2 and "provide text or --file" in err, f"{code} {err[-60:]}")

code, out, err = run(["x" * 3000, "-o", os.path.join("oversize", "deep", "huge.png")])
check("rejects oversize", code == 1 and err.startswith("FAIL store") and not os.path.exists(art("oversize")), f"{code} {err[:80]}")

code, out, err = run(["", "-f", "secret.txt", "-o", "emptyboth.png"])
check("rejects empty text with file", code == 2 and "not both" in err, f"{code} {err[-60:]}")

short = "ab"
code, out, err = run([short, "-o", "short.png"])
check("short payload regular 21 modules", code == 0 and image_size(art("short.png"))[0] == 290 and decode(art("short.png")) == short, f"{code} {err}")
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
with open(art("titled.html"), encoding="utf-8") as fh:
    titled = fh.read()
check("title escaped in page", code == 0 and "&lt;b&gt;&amp;&quot;x&quot;&lt;/b&gt;" in titled, titled[:120])
check("title never encoded", decode(art("titled.png")) == secret, str(decode(art("titled.png"))))

code, out, err = run(["first", "-o", "over.png"])
code, out, err = run(["second", "-o", "over.png", "--scale", "12"])
check("overwrite replaces the file", code == 0 and decode(art("over.png")) == "second", f"{code} {err}")

with open(art("over.png"), "rb") as fh:
    over_before = fh.read()
os.chmod(art("over.png"), stat.S_IREAD)
code, out, err = run(["third", "-o", "over.png"])
check(
    "read-only output is refused",
    code == 1 and out == "" and err.startswith("FAIL write") and "read-only" in err,
    f"{code} {out!r} {err[:90]}",
)
with open(art("over.png"), "rb") as fh:
    over_after = fh.read()
check(
    "read-only output keeps the old image",
    over_after == over_before and decode(art("over.png")) == "second",
    f"{len(over_before)} {len(over_after)}",
)
check(
    "read-only failure leaves no temp file",
    [name for name in os.listdir(WORK) if name.endswith(".tmp")] == [],
    str(os.listdir(WORK)),
)
os.chmod(art("over.png"), stat.S_IREAD | stat.S_IWRITE)

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

code, out, err = run(["x", "-o", "clash.html", "--html"])
check(
    "page equal to the png is refused",
    code == 2 and "page path equals the PNG path" in err and out == "" and not os.path.exists(art("clash.html")),
    f"{code} {out!r} {err[:90]}",
)

code, out, err = run(["x", "-o", "a#b.png", "--html"])
with open(art("a#b.html"), encoding="utf-8") as fh:
    hash_page = fh.read()
check(
    "page encodes a # in the image name",
    code == 0 and 'src="a%23b.png"' in hash_page and os.path.exists(art("a#b.png")),
    f"{code} {hash_page[:90]}",
)

code, out, err = run(["x", "--title", "t"])
check(
    "title without html is refused",
    code == 2 and "--title requires --html" in err and out == "",
    f"{code} {out!r} {err[-90:]}",
)

with open(art("huge.txt"), "wb") as fh:
    fh.write(b"a" * (1024 * 1024 + 1))
code, out, err = run(["-f", "huge.txt", "-o", "huge-bound.png"])
check(
    "rejects an input file above 1 MiB",
    code == 1 and err.startswith("FAIL read") and "1 MiB" in err and not os.path.exists(art("huge-bound.png")),
    f"{code} {err[:90]}",
)

with open(art("blocker"), "w", encoding="utf-8") as fh:
    fh.write("x")
code, out, err = run(["x", "-o", os.path.join("blocker", "hidden.png")])
check("blocks output parent with FAIL write", code == 1 and out == "" and err.startswith("FAIL write") and "Traceback" not in err, f"{code} {err[:90]}")

os.makedirs(art("blocked.html"), exist_ok=True)
code, out, err = run(["x", "-o", "blocked.png", "--html"])
check("blocks page with FAIL write", code == 1 and out == "" and err.startswith("FAIL write") and "Traceback" not in err, f"{code} {err[:90]}")
check("page failure keeps the PNG", os.path.exists(art("blocked.png")), "png removed")
check(
    "page failure leaves no temp file",
    [name for name in os.listdir(WORK) if name.endswith(".tmp")] == [],
    str(os.listdir(WORK)),
)

exotic = "bad\ud800.txt" if os.name == "nt" else "bad\udcff.txt"
exotic_out = "out\ud800.png" if os.name == "nt" else "out\udcff.png"
if sys.platform == "darwin":
    # APFS refuses a name that is not valid UTF-8, so the case cannot be created.
    print("SKIP a name that is not valid unicode is used exactly :: the file system refuses the name")
else:
    with open(art(exotic), "wb") as fh:
        fh.write(b"exotic")
    code, out, err = run(["-f", exotic, "-o", exotic_out])
    check(
        "a name that is not valid unicode is used exactly",
        code == 0 and decode(art(exotic_out)) == "exotic" and err == "",
        f"{code} {err[:90]}",
    )

code, out, err = run(["x" * 2300, "-o", "extreme.png", "--scale", "100", "--border", "16"])
size, complete = png_probe(art("extreme.png"))
check(
    "scale 100 border 16 gives a 20900 pixel image",
    code == 0 and size == (20900, 20900) and err == "",
    f"{code} {size} {err[:60]}",
)
check(
    "the largest image ends with IEND",
    complete,
    str(complete),
)
os.remove(art("extreme.png"))

trunc = art("trunc.png")
with open(art("plain.png"), "rb") as src, open(trunc, "wb") as dst:
    dst.write(src.read(100))
size, complete = png_probe(trunc)
full_size, full_complete = png_probe(art("plain.png"))
check("png tail detects truncation", size is not None and not complete and full_complete, f"{size} {complete} {full_complete}")

term_dir = art("term")
os.makedirs(term_dir)
code, out, err = run_bytes(["ab"], term_dir)
text = out.decode("utf-8", "replace")
image, result = term_decode(text)
check("term exits 0 with no stderr", code == 0 and err == b"", f"{code} {err[:60]!r}")
check("term writes nothing to disk", os.listdir(term_dir) == [], str(os.listdir(term_dir)))
check("term paints every line", image is not None, text[:40])
check("term decodes to the payload", result is not None and result.text == "ab", str(result))
check(
    "term keeps a square white quiet zone",
    image is not None
    and image.width == 29
    and image.height == 30
    and image.getextrema() == (0, 255)
    and image.getpixel((0, 0)) == 255,
    f"{image.size if image else None} {image.getextrema() if image else None}",
)

zero_dir = art("term-zero")
os.makedirs(zero_dir)
code, out, err = run_bytes(["--border", "0", "ab"], zero_dir)
image, result = term_decode(out.decode("utf-8", "replace"))
check(
    "term border 0 gives a bare 21-module grid",
    code == 0
    and image is not None
    and image.width == 21
    and result is not None
    and result.text == "ab"
    and os.listdir(zero_dir) == [],
    f"{code} {image.size if image else None} {os.listdir(zero_dir)}",
)

big_dir = art("term-big")
os.makedirs(big_dir)
code, out, err = run_bytes(["x" * 3000], big_dir)
check(
    "term oversize writes nothing",
    code == 1 and err.startswith(b"FAIL store") and os.listdir(big_dir) == [],
    f"{code} {err[:60]!r}",
)

save_dir = art("save")
os.makedirs(save_dir)
code, out, err = run_bytes(["--save", "ab"], save_dir)
saved = sorted(os.listdir(save_dir))
check(
    "save writes qr.png and prints the path",
    code == 0 and saved == ["qr.png"] and out.decode("utf-8").strip().endswith("qr.png"),
    f"{code} {saved} {out[:60]!r}",
)
check(
    "save writes a decodable PNG",
    saved == ["qr.png"] and decode(os.path.join(save_dir, "qr.png")) == "ab",
    str(saved),
)

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
