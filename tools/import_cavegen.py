"""Converts a game folder from JHawk's CaveGen (`CaveGen/files/<name>`) into CaveGen3's layout.

    python tools/import_cavegen.py <CaveGen/files/name> <game>

writes `data/assets/<game>/`. The CaveGen folder needs `caveinfo/`, `units/` and `arc/`, and
optionally `enemytex/` and `resulttex/`, as described in CaveGen's Adding_New_Romhacks.md
(SZS archives extracted and BTI images decoded to PNG).

Caveinfo files are renamed to the exact case `data/resources/caveinfo_config.txt` uses, since
the web build and Linux are case sensitive. Only files CaveGen3 reads are copied.
"""

import shutil
import sys
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


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    src, game = Path(sys.argv[1]), sys.argv[2]
    out = ROOT / "data/assets" / game
    if out.exists():
        sys.exit(f"{out} already exists; delete it first to re-import")
    for needed in ["caveinfo", "units", "arc"]:
        if not (src / needed).is_dir():
            sys.exit(f"{src / needed} is missing")

    names = config_filenames(game)
    if not names:
        print(f"warning: no caves for '{game}' in data/resources/caveinfo_config.txt yet")
    found = set()
    for f in sorted((src / "caveinfo").glob("*.txt")):
        name = names.get(f.name.lower(), f.name)
        found.add(name.lower())
        copy(f, out / "caveinfo" / name)
    for missing in sorted(set(names) - found):
        print(f"warning: caveinfo_config.txt lists {names[missing]}, which isn't in {src / 'caveinfo'}")

    for f in sorted((src / "units").glob("*.txt")):
        copy(f, out / "unitfiles" / f.name)

    units = 0
    for unit in sorted(p for p in (src / "arc").iterdir() if p.is_dir()):
        texts = unit / "texts.d"
        if not texts.is_dir():
            continue
        units += 1
        for txt in ["layout.txt", "route.txt", "waterbox.txt"]:
            if (texts / txt).is_file():
                copy(texts / txt, out / "mapunits" / unit.name / "texts" / txt)
        png = unit / "arc.d" / "texture.bti.png"
        if png.is_file():
            copy(png, out / "mapunits" / unit.name / "arc" / "texture.png")

    icons = {"teki": src / "enemytex/arc.d", "treasures": src / "resulttex/us/arc.d"}
    counts = {}
    for kind, folder in icons.items():
        counts[kind] = 0
        if not folder.is_dir():
            continue
        for p in sorted(folder.iterdir()):
            # Either <name>/texture.bti.png or <name>.bti.png, depending on the extraction tool.
            if p.is_dir() and (p / "texture.bti.png").is_file():
                name, png = p.name, p / "texture.bti.png"
            elif p.name.endswith(".bti.png"):
                name, png = p.name[: -len(".bti.png")], p
            else:
                continue
            copy(png, out / kind / f"{name}.png")
            counts[kind] += 1

    for cfg in ["otakara_config.txt", "item_config.txt"]:
        if (src / cfg).is_file():
            copy(src / cfg, out / cfg)

    print(f"{out}: {len(found)} caveinfo files, {units} map units, {counts['teki']} teki icons, "
          f"{counts['treasures']} treasure icons")


if __name__ == "__main__":
    main()
