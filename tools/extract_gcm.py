"""Extracts files from a GameCube disc image (GCM/ISO) by its file table.

    python tools/extract_gcm.py <iso> <outdir> [prefix ...]

Writes <outdir>/files/<path> for files whose path starts with one of the prefixes (all files if
none; `user message` is what tools/import_game_files.py and the cave names need), and
<outdir>/sys/main.dol and boot.bin. File names are Shift-JIS; characters Windows can't use in
a file name become `_`.
"""
import struct
import sys
from pathlib import Path


def main(iso, outdir, *prefixes):
    out = Path(outdir)
    f = open(iso, "rb")
    boot = f.read(0x440)
    dol_off, fst_off, fst_size = struct.unpack(">III", boot[0x420:0x42C])
    (out / "sys").mkdir(parents=True, exist_ok=True)
    (out / "sys/boot.bin").write_bytes(boot)
    # DOL size: max(offset + size) over its 18 sections.
    f.seek(dol_off)
    hdr = f.read(0x100)
    offs = struct.unpack(">18I", hdr[0:0x48])
    sizes = struct.unpack(">18I", hdr[0x90:0xD8])
    dol_len = max(o + s for o, s in zip(offs, sizes) if s)
    f.seek(dol_off)
    (out / "sys/main.dol").write_bytes(f.read(dol_len))
    f.seek(fst_off)
    fst = f.read(fst_size)
    n = struct.unpack(">I", fst[8:12])[0]
    names = fst[n * 12:]

    def name(i):
        off = struct.unpack(">I", fst[i * 12:i * 12 + 4])[0] & 0xFFFFFF
        return names[off:names.index(b"\0", off)].decode("cp932", errors="replace").translate({ord(c): "_" for c in '?*:<>|"�'})

    count = 0
    # Walk the directory tree: entry i for a dir holds parent and "next" index.
    stack = [(n, "")]  # (end index, path prefix)
    i = 1
    while i < n:
        while stack and i >= stack[-1][0]:
            stack.pop()
        word, a, b = struct.unpack(">III", fst[i * 12:i * 12 + 12])
        path = stack[-1][1] + name(i)
        if word >> 24:
            stack.append((b, path + "/"))
        elif not prefixes or any(path.lower().startswith(p.lower()) for p in prefixes):
            dst = out / "files" / path
            dst.parent.mkdir(parents=True, exist_ok=True)
            f.seek(a)
            dst.write_bytes(f.read(b))
            count += 1
        i += 1
    print(f"{count} files, game id {boot[:6].decode(errors='replace')}, title {boot[0x20:0x60].split(b'\0')[0].decode(errors='replace')}")


if __name__ == "__main__":
    main(*sys.argv[1:])
