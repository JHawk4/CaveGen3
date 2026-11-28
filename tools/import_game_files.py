"""Imports a romhack from the game's own files (an extracted ISO, or a patch folder that holds
replacement files) into CaveGen3's layout.

    python tools/import_game_files.py <root> <game> [--base <game>]

<root> is the folder that holds `user/` (for an extracted ISO, its `files` folder). Writes
`data/assets/<game>/`. Needs Wiimms SZS Tools (`wszst` and `wimgt`) on PATH to unpack the
`.szs` archives and decode BTI images.

With `--base pikmin2`, unit files the caveinfo names and map units the unit files use, when <root>
doesn't contain them, are copied from `data/assets/pikmin2`. Romhacks shipped as file replacements (Pikmin 216) only include the
units they change.

Caveinfo files are renamed to the exact case `data/resources/caveinfo_config.txt` uses. Only
files CaveGen3 reads are written.
"""

import argparse
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def config_filenames(game):
    names = {}
    for line in (ROOT / "data/resources/caveinfo_config.txt").read_text(encoding="utf-8").splitlines():
        parts = [p.strip() for p in line.split(",")]
        if len(parts) >= 4 and parts[0] == game:
            names[parts[3].lower()] = parts[3]
    return names


def copy(src, dst):
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)


def unpack(szs, dest):
    """Extracts an SZS (Yaz0-compressed RARC) archive into `dest`.

    Two kinds wszst can't read are decompressed here first: Yay0 archives, which the game also
    reads (Hallow's Haunt's enemytex), and RARCs whose header claims a few more bytes than the
    data holds (some of Hallow's Haunt's map units), which wszst lists as empty. Those are
    padded with zeros to the size their header gives."""
    import struct

    data = szs.read_bytes()
    if data[:4] != b"Yay0":
        subprocess.run(["wszst", "extract", "-q", "-o", szs.as_posix(), "-d", dest.as_posix()], check=True)
        if dest.is_dir() and any(p.is_file() for p in dest.rglob("*")):
            return
    rarc = yay0(data) if data[:4] == b"Yay0" else yaz0(data) if data[:4] == b"Yaz0" else data
    declared = struct.unpack(">I", rarc[4:8])[0] if rarc[:4] == b"RARC" else 0
    rarc += bytes(max(0, declared - len(rarc)))
    tmp = dest.parent / (dest.name + ".rarc")
    tmp.parent.mkdir(parents=True, exist_ok=True)
    tmp.write_bytes(rarc)
    subprocess.run(["wszst", "extract", "-q", "-o", tmp.as_posix(), "-d", dest.as_posix()], check=True)


def yaz0(data):
    """Decompresses Yaz0: a code byte per 8 chunks, each a literal byte or a back reference."""
    import struct

    size, pos, out = struct.unpack(">I", data[4:8])[0], 16, bytearray()
    while len(out) < size:
        code = data[pos]
        pos += 1
        for bit in range(8):
            if len(out) >= size:
                break
            if code & (0x80 >> bit):
                out.append(data[pos])
                pos += 1
                continue
            b1, b2 = data[pos], data[pos + 1]
            pos += 2
            dist, n = ((b1 & 0xF) << 8 | b2) + 1, b1 >> 4
            if n == 0:
                n = data[pos] + 0x12
                pos += 1
            else:
                n += 2
            for _ in range(n):
                out.append(out[-dist])
    return bytes(out)


def yay0(data):
    """Decompresses Yay0 (JKRDecomp::decodeSZP): a bit mask stream, then 16-bit back
    references at the link offset, then literal and long-count bytes at the chunk offset."""
    import struct

    size, link, chunk = struct.unpack(">III", data[4:16])
    mask_pos, out, mask, bits = 16, bytearray(), 0, 0
    while len(out) < size:
        if bits == 0:
            mask, bits = struct.unpack(">I", data[mask_pos:mask_pos + 4])[0], 32
            mask_pos += 4
        if mask & 0x80000000:
            out.append(data[chunk])
            chunk += 1
        else:
            v = struct.unpack(">H", data[link:link + 2])[0]
            link += 2
            dist, n = (v & 0xFFF) + 1, v >> 12
            if n == 0:
                n = data[chunk] + 0x12
                chunk += 1
            else:
                n += 2
            for _ in range(n):
                out.append(out[-dist])
        mask, bits = (mask << 1) & 0xFFFFFFFF, bits - 1
    return bytes(out)


def bti_to_png(bti, png):
    png.parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(["wimgt", "decode", "-q", "-o", bti.as_posix(), "-d", png.as_posix()], capture_output=True)
    # wimgt fails on some romhack BTIs, and skips others silently without writing anything.
    if (r.returncode != 0 or not png.is_file()) and not decode_bti(bti, png):
        print(f"warning: couldn't decode {bti}: {r.stderr.decode(errors='replace').strip()}")


def decode_bti(bti, png):
    """Fallback for the BTI files wimgt refuses or skips (some romhacks write unusual header
    fields, such as a palette format on a texture with no palette). Handles I4/I8/IA4/IA8,
    RGB565/RGB5A3/RGBA8 and CMPR and writes an RGBA PNG. Returns False for other
    formats."""
    import struct
    import zlib

    data = bti.read_bytes()
    fmt = data[0]
    w, h = struct.unpack(">HH", data[2:6])
    offset = struct.unpack(">I", data[0x1C:0x20])[0]
    pixels = [[(0, 0, 0, 0)] * w for _ in range(h)]

    def put(x, y, px):
        if y < h and x < w:
            pixels[y][x] = px

    def rgb565(c):
        r, g, b = c >> 11, (c >> 5) & 63, c & 31
        return (r * 255 // 31, g * 255 // 63, b * 255 // 31)

    pos = offset
    if fmt == 0xE:  # CMPR: 8x8 blocks of four 4x4 DXT1 sub-blocks
        for by in range(0, h, 8):
            for bx in range(0, w, 8):
                for sub in range(4):
                    c0, c1 = struct.unpack(">HH", data[pos:pos + 4])
                    a, b = rgb565(c0), rgb565(c1)
                    if c0 > c1:
                        pal = [a + (255,), b + (255,),
                               tuple((2 * p + q) // 3 for p, q in zip(a, b)) + (255,),
                               tuple((p + 2 * q) // 3 for p, q in zip(a, b)) + (255,)]
                    else:
                        pal = [a + (255,), b + (255,),
                               tuple((p + q) // 2 for p, q in zip(a, b)) + (255,), (0, 0, 0, 0)]
                    for y in range(4):
                        row = data[pos + 4 + y]
                        for x in range(4):
                            put(bx + (sub % 2) * 4 + x, by + (sub // 2) * 4 + y, pal[(row >> (6 - 2 * x)) & 3])
                    pos += 8
    elif fmt == 6:  # RGBA8: 4x4 blocks, 32 bytes of AR pairs then 32 of GB pairs
        for by in range(0, h, 4):
            for bx in range(0, w, 4):
                for k in range(16):
                    a, r = data[pos + 2 * k], data[pos + 2 * k + 1]
                    g, b = data[pos + 32 + 2 * k], data[pos + 33 + 2 * k]
                    put(bx + k % 4, by + k // 4, (r, g, b, a))
                pos += 64
    else:
        # (block width, block height, bits per pixel)
        blocks = {0: (8, 8, 4), 1: (8, 4, 8), 2: (8, 4, 8), 3: (4, 4, 16), 4: (4, 4, 16), 5: (4, 4, 16)}
        if fmt not in blocks:
            return False
        bw, bh, bpp = blocks[fmt]
        for by in range(0, h, bh):
            for bx in range(0, w, bw):
                for y in range(bh):
                    for x in range(bw):
                        if fmt == 0:  # I4
                            byte = data[pos + (y * bw + x) // 2]
                            i, a = (byte >> 4 if x % 2 == 0 else byte & 0xF) * 17, 255
                        elif fmt == 1:  # I8
                            i, a = data[pos + y * bw + x], 255
                        elif fmt == 2:  # IA4
                            byte = data[pos + y * bw + x]
                            i, a = (byte & 0xF) * 17, (byte >> 4) * 17
                        elif fmt == 3:  # IA8
                            a, i = data[pos + 2 * (y * bw + x)], data[pos + 2 * (y * bw + x) + 1]
                        else:
                            c = struct.unpack(">H", data[pos + 2 * (y * bw + x):pos + 2 * (y * bw + x) + 2])[0]
                            if fmt == 4:  # RGB565
                                put(bx + x, by + y, rgb565(c) + (255,))
                            elif c & 0x8000:  # RGB5A3, opaque: RGB555
                                put(bx + x, by + y, ((c >> 10 & 31) * 255 // 31, (c >> 5 & 31) * 255 // 31, (c & 31) * 255 // 31, 255))
                            else:  # RGB5A3, translucent: A3 RGB444
                                put(bx + x, by + y, ((c >> 8 & 15) * 17, (c >> 4 & 15) * 17, (c & 15) * 17, (c >> 12 & 7) * 255 // 7))
                            continue
                        put(bx + x, by + y, (i, i, i, a))
                pos += bw * bh * bpp // 8
    raw = b"".join(b"\0" + bytes(c for px in row for c in px) for row in pixels)

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

    png.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
                    + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))
    return True


def icons(archive, out_dir, tmp):
    """Each `<name>/texture.bti` in an icon archive becomes `<out_dir>/<name>.png`."""
    if not archive.is_file():
        return 0
    unpack(archive, tmp)
    n = 0
    for bti in sorted(tmp.glob("*/texture.bti")):
        bti_to_png(bti, out_dir / f"{bti.parent.name}.png")
        n += 1
    return n


def units_named(unitfile):
    text = unitfile.read_bytes().decode("shift_jis", errors="replace")
    return re.findall(r"^\s*(\S+)\s*#\s*foldername", text, re.M)


def unitfiles_named(caveinfo):
    """Unit file names a caveinfo file refers to (`{f008} -1 <name> # UnitFile`)."""
    text = caveinfo.read_bytes().decode("shift_jis", errors="replace")
    return re.findall(r"\{f008\}\s+-?\d+\s+(\S+)", text)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("root", type=Path)
    ap.add_argument("game")
    ap.add_argument("--base", help="game in data/assets to take missing unit files and map units from")
    args = ap.parse_args()

    mapunits = args.root / "user/Mukki/mapunits"
    out = ROOT / "data/assets" / args.game
    if out.exists():
        raise SystemExit(f"{out} already exists; delete it first to re-import")
    for needed in ["caveinfo", "units", "arc"]:
        if not (mapunits / needed).is_dir():
            raise SystemExit(f"{mapunits / needed} is missing")

    names = config_filenames(args.game)
    if not names:
        print(f"warning: no caves for '{args.game}' in data/resources/caveinfo_config.txt yet")
    found = set()
    for f in sorted((mapunits / "caveinfo").glob("*.txt")):
        name = names.get(f.name.lower(), f.name)
        found.add(name.lower())
        copy(f, out / "caveinfo" / name)
    for missing in sorted(set(names) - found):
        print(f"warning: caveinfo_config.txt lists {names[missing]}, which isn't in {mapunits / 'caveinfo'}")

    for f in sorted((mapunits / "units").glob("*.txt")):
        copy(f, out / "unitfiles" / f.name)

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        units = 0
        for unit in sorted(p for p in (mapunits / "arc").iterdir() if p.is_dir()):
            if not (unit / "texts.szs").is_file():
                continue
            units += 1
            texts = tmp / unit.name / "texts"
            unpack(unit / "texts.szs", texts)
            for txt in ["layout.txt", "route.txt", "waterbox.txt"]:
                if (texts / txt).is_file():
                    copy(texts / txt, out / "mapunits" / unit.name / "texts" / txt)
            if (unit / "arc.szs").is_file():
                arc = tmp / unit.name / "arc"
                unpack(unit / "arc.szs", arc)
                if (arc / "texture.bti").is_file():
                    bti_to_png(arc / "texture.bti", out / "mapunits" / unit.name / "arc" / "texture.png")

        teki = icons(args.root / "user/Yamashita/enemytex/arc.szs", out / "teki", tmp / "enemytex")
        treasures = icons(args.root / "user/Matoba/resulttex/us/arc.szs", out / "treasures", tmp / "resulttex")

        pellets = args.root / "user/Abe/Pellet/us/pelletlist_us.szs"
        if pellets.is_file():
            unpack(pellets, tmp / "pellets")
            for cfg in ["otakara_config.txt", "item_config.txt"]:
                if (tmp / "pellets" / cfg).is_file():
                    copy(tmp / "pellets" / cfg, out / cfg)

    borrowed = borrowed_files = 0
    if args.base:
        # Unit files the caveinfo names but the romhack doesn't include are the base game's.
        base_units = ROOT / "data/assets" / args.base / "unitfiles"
        have = {p.name.lower() for p in (out / "unitfiles").iterdir()}
        for name in sorted({u for f in (out / "caveinfo").glob("*.txt") for u in unitfiles_named(f)}):
            src = next((p for p in base_units.iterdir() if p.name.lower() == name.lower()), None)
            if name.lower() not in have and src is not None:
                copy(src, out / "unitfiles" / src.name)
                borrowed_files += 1

        base = ROOT / "data/assets" / args.base / "mapunits"
        wanted = {u for f in (out / "unitfiles").glob("*.txt") for u in units_named(f)}
        have = {p.name.lower() for p in (out / "mapunits").iterdir()}
        for name in sorted(wanted):
            if name.lower() in have:
                continue
            src = next((p for p in base.iterdir() if p.name.lower() == name.lower()), None)
            if src is None:
                print(f"warning: map unit {name} is in neither {args.root} nor {base}")
                continue
            shutil.copytree(src, out / "mapunits" / src.name)
            borrowed += 1

    print(f"{out}: {len(found)} caveinfo files, {units} map units"
          f"{f' (+{borrowed} and {borrowed_files} unit files from {args.base})' if args.base else ''}, {teki} teki icons, {treasures} treasure icons")


if __name__ == "__main__":
    main()
