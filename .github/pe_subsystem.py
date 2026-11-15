"""Fail unless a PE file is a console application (subsystem 3)."""
import struct
import sys


def subsystem(path):
    with open(path, "rb") as fh:
        head = fh.read(0x400)
    offset = struct.unpack_from("<I", head, 0x3C)[0]
    if head[offset:offset + 4] != b"PE\0\0":
        raise SystemExit("not a PE file")
    return struct.unpack_from("<H", head, offset + 0x5C)[0]


def main(argv):
    if len(argv) != 1:
        raise SystemExit("usage: pe_subsystem.py FILE")
    value = subsystem(argv[0])
    print(f"subsystem {value}")
    return 0 if value == 3 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
