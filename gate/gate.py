import os
import subprocess
import sys

SKILL = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(os.path.dirname(os.path.abspath(__file__)), "out")
PY = sys.executable
fails = []

import shutil  # noqa: E402

shutil.rmtree(WORK, ignore_errors=True)
os.makedirs(WORK, exist_ok=True)


def run(args):
    p = subprocess.run([PY, os.path.join(SKILL, "make-qr.py")] + args, cwd=WORK, capture_output=True, text=True)
    return p.returncode, p.stdout.strip(), p.stderr.strip()


def decode(path):
    import zxingcpp
    from PIL import Image
    r = zxingcpp.read_barcode(Image.open(path))
    return r.text if r else None


def check(name, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + name + ("" if cond else " :: " + detail))
    if not cond:
        fails.append(name)


os.makedirs(WORK, exist_ok=True)
url = "https://asusvivos14.forest-piranha.ts.net/ntfy-proj?x=1"

code, out, err = run([url, "-o", "plain.png"])
lines = out.splitlines()
check("text exit 0", code == 0, err)
check("prints one absolute path", len(lines) == 1 and os.path.isabs(lines[0]), out)
check("decode matches", decode(os.path.join(WORK, "plain.png")) == url, str(decode(os.path.join(WORK, "plain.png"))))

secret = "tk_0123456789abcdef0123456789abcdef"
with open(os.path.join(WORK, "secret.txt"), "w", encoding="utf-8") as fh:
    fh.write("  " + secret + "\n")
code, out, err = run(["-f", "secret.txt", "-o", "deep\\secret.png", "--scale", "12", "--html", "--title", "phone token"])
paths = out.splitlines()
check("file exit 0", code == 0, err)
check("two paths png then html", len(paths) == 2 and paths[0].endswith(".png") and paths[1].endswith(".html"), out)
check("strips file whitespace", decode(os.path.join(WORK, "deep", "secret.png")) == secret, str(decode(os.path.join(WORK, "deep", "secret.png"))))
page = open(os.path.join(WORK, "deep", "secret.html"), encoding="utf-8").read()
check("viewer references png", 'src="secret.png"' in page, page[:80])
check("viewer hides text", secret not in page and "tk_" not in page, "text leaked")

from PIL import Image  # noqa: E402

img = Image.open(os.path.join(WORK, "deep", "secret.png"))
check("scale 12 pixels", img.width == img.height and img.width % 12 == 0, f"{img.width}x{img.height}")
img = Image.open(os.path.join(WORK, "plain.png"))
check("default scale 10 pixels", img.width == img.height and img.width % 10 == 0, f"{img.width}x{img.height}")

code, out, err = run(["x", "-f", "secret.txt", "-o", "both.png"])
check("rejects text plus file", code == 2 and "not both" in err, f"{code} {err[-60:]}")

code, out, err = run([""])
check("rejects empty text", code == 2 and "empty input" in err, f"{code} {err[-60:]}")

code, out, err = run(["x" * 3000, "-o", "huge.png"])
check("rejects oversize", code == 1 and err.startswith("FAIL store") and not os.path.exists(os.path.join(WORK, "huge.png")), f"{code} {err[:80]}")

code, out, err = run(["x", "--scale", "0", "-o", "bad.png"])
check("rejects scale 0", code == 2 and "--scale must be 1-100" in err, f"{code} {err[-60:]}")
code, out, err = run(["x", "--border", "17", "-o", "bad2.png"])
check("rejects border 17", code == 2 and "--border must be 0-16" in err, f"{code} {err[-60:]}")

print(f"GATE failures={len(fails)}")
sys.exit(1 if fails else 0)
