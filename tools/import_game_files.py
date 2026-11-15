"""Imports a romhack from the game's own files (an extracted ISO, or a patch folder that holds
replacement files) into CaveGen3's layout.

    python tools/import_game_files.py <root> <game> [--base <game>]

<root> is the folder that holds `user/` (for an extracted ISO, its `files` folder). Writes
`data/assets/<game>/`. Needs Wiimms SZS Tools (`wszst` and `wimgt`) on PATH to unpack the
`.szs` archives and decode BTI images.

With `--base pikmin2`, map units the unit files use but <root> doesn't contain are copied from
`data/assets/pikmin2`. Romhacks shipped as file replacements (Pikmin 216) only include the
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
    """Extracts an SZS (Yaz0-compressed RARC) archive into `dest`."""
    subprocess.run(["wszst", "extract", "-q", "-o", szs.as_posix(), "-d", dest.as_posix()], check=True)


def bti_to_png(bti, png):
    png.parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(["wimgt", "decode", "-q", "-o", bti.as_posix(), "-d", png.as_posix()], capture_output=True)
    # wimgt fails on some romhack BTIs, and skips others silently without writing anything.
    if (r.returncode != 0 or not png.is_file()) and not decode_bti(bti, png):
        print(f"warning: couldn't decode {bti}: {r.stderr.decode(errors='replace').strip()}")


def decode_bti(bti, png):
    """Fallback for the BTI files wimgt refuses or skips (some romhacks write unusual header
    fields). Handles I4/I8/IA4/IA8 and CMPR and writes an RGBA PNG. Returns False for other
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
    else:
        # (block width, block height, bits per pixel)
        blocks = {0: (8, 8, 4), 1: (8, 4, 8), 2: (8, 4, 8), 3: (4, 4, 16)}
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
                        else:  # IA8
                            a, i = data[pos + 2 * (y * bw + x)], data[pos + 2 * (y * bw + x) + 1]
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


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("root", type=Path)
    ap.add_argument("game")
    ap.add_argument("--base", help="game in data/assets to take missing map units from")
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

    borrowed = 0
    if args.base:
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
          f"{f' (+{borrowed} from {args.base})' if args.base else ''}, {teki} teki icons, {treasures} treasure icons")


if __name__ == "__main__":
    main()
