"""Builds the web page into `site/` (or the folder given as the first argument).

    cargo build --release -p cavegen3-web --target wasm32-unknown-unknown
    python tools/build_site.py [out_dir]

The page is `web/`. Game data goes in `data/<name>.bin.gz` bundles (the format `parse_bundle`
in crates/cavegen3-web reads), loaded one game at a time. Image file names are lowercased, since the
game ignores case and web servers don't; the page asks for lowercase names.
"""

import gzip
import shutil
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "data"
WASM = ROOT / "target/wasm32-unknown-unknown/release/cavegen3_web.wasm"


def bundle(files):
    out = bytearray()
    for rel, path in files:
        data = path.read_bytes()
        name = rel.encode("utf-8")
        out += struct.pack("<I", len(name)) + name + struct.pack("<I", len(data)) + data
    return gzip.compress(bytes(out), compresslevel=9, mtime=0)


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "site"
    if not WASM.is_file():
        sys.exit(f"{WASM} is missing; build it first (see this script's docstring)")
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(ROOT / "web", out)
    shutil.copyfile(WASM, out / "cavegen3_web.wasm")
    (out / ".nojekyll").write_text("")

    (out / "data").mkdir()
    config = DATA / "resources/caveinfo_config.txt"
    (out / "data/resources.bin.gz").write_bytes(bundle([("resources/caveinfo_config.txt", config)]))

    games = []
    for game_dir in sorted(p for p in (DATA / "assets").iterdir() if p.is_dir()):
        game = game_dir.name
        texts = sorted(p for p in game_dir.rglob("*.txt"))
        files = [(p.relative_to(DATA).as_posix(), p) for p in texts]
        (out / f"data/{game}.bin.gz").write_bytes(bundle(files))
        games.append(game)

        img = out / "img" / game
        for unit in sorted((game_dir / "mapunits").iterdir()):
            png = unit / "arc/texture.png"
            if png.is_file():
                (img / "mapunits").mkdir(parents=True, exist_ok=True)
                shutil.copyfile(png, img / "mapunits" / f"{unit.name.lower()}.png")
        for kind in ["teki", "treasures"]:
            if (game_dir / kind).is_dir():
                (img / kind).mkdir(parents=True)
                for png in (game_dir / kind).glob("*.png"):
                    shutil.copyfile(png, img / kind / png.name.lower())

    (out / "img/special").mkdir(parents=True)
    for png in (DATA / "resources/enemytex_special").iterdir():
        shutil.copyfile(png, out / "img/special" / png.name.lower())
    (out / "data/games.txt").write_text("\n".join(games) + "\n")

    size = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    print(f"{out}: {len(games)} games ({', '.join(games)}), {size / 1e6:.1f} MB")


if __name__ == "__main__":
    main()
