"""Check the PE headers of a Windows binary: console subsystem, static C runtime.

Subsystem 3 is a console application. A GUI build (2) starts with no console,
so its output goes nowhere. The import table must not name VCRUNTIME140,
MSVCP140 or the api-ms-win-crt-* apisets: those come from the Visual C++
redistributable, which a folder copy cannot install.

Usage: python .github/pe_subsystem.py FILE [FILE ...]
"""
import struct
import sys

CONSOLE = 3
BANNED = ("vcruntime", "msvcp", "api-ms-win-crt")


def headers(path):
    with open(path, "rb") as fh:
        data = fh.read()
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise ValueError("not a PE file")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("not a PE file")
    optional = pe + 24
    magic = struct.unpack_from("<H", data, optional)[0]
    if magic == 0x10B:
        count_at, dirs_at = 0x5C, 0x60
    elif magic == 0x20B:
        count_at, dirs_at = 0x6C, 0x70
    else:
        raise ValueError(f"unknown optional header {magic:#x}")
    sizes = struct.unpack_from("<H", data, pe + 20)[0]
    subsystem = struct.unpack_from("<H", data, optional + 0x44)[0]
    count = min(struct.unpack_from("<I", data, optional + count_at)[0], 16)
    dirs = [struct.unpack_from("<II", data, optional + dirs_at + 8 * i) for i in range(count)]
    return data, optional + sizes, dirs, subsystem


def section_count(data, pe):
    return struct.unpack_from("<H", data, pe + 6)[0]


def offset_of(data, sections, count, rva):
    for i in range(count):
        head = sections + 40 * i
        vsize, va, raw_size, raw_ptr = struct.unpack_from("<IIII", data, head + 8)
        if va <= rva < va + max(vsize, raw_size):
            return rva - va + raw_ptr
    return None


def imports(data, sections, count, dirs):
    names = []
    if len(dirs) < 2:
        return names
    rva, size = dirs[1]
    if rva == 0 or size == 0:
        return names
    offset = offset_of(data, sections, count, rva)
    while offset is not None and offset + 20 <= len(data):
        entry = data[offset:offset + 20]
        if entry == b"\0" * 20:
            break
        name_rva = struct.unpack_from("<I", entry, 12)[0]
        name_offset = offset_of(data, sections, count, name_rva)
        if name_offset is None:
            break
        end = data.find(b"\0", name_offset)
        if end < 0:
            break
        names.append(data[name_offset:end].decode("ascii", "replace"))
        offset += 20
    return names


def check(path):
    try:
        data, sections, dirs, subsystem = headers(path)
        pe = struct.unpack_from("<I", data, 0x3C)[0]
        names = imports(data, sections, section_count(data, pe), dirs)
    except (ValueError, struct.error, IndexError) as err:
        print(f"FAIL {path}: {err}")
        return 1
    bad = [name for name in names if name.lower().startswith(BANNED)]
    print(f"{path}: subsystem {subsystem}, {len(names)} imports")
    failed = 0
    if subsystem != CONSOLE:
        print(f"FAIL {path}: subsystem {subsystem}, want {CONSOLE}")
        failed = 1
    if bad:
        print(f"FAIL {path}: dynamic C runtime {', '.join(bad)}")
        failed = 1
    return failed


def main(argv):
    if not argv:
        print("usage: pe_subsystem.py FILE [FILE ...]", file=sys.stderr)
        return 2
    return 1 if sum(check(path) for path in argv) else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
