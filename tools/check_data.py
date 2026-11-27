"""Checks that every file the game data refers to exists with exactly that name.

    python tools/check_data.py [--fix] [game ...]

The game's own file system ignores case, so romhacks sometimes refer to `SR7_Racetrack_toy.txt`
as `SR7_racetrack_toy.txt`. Windows hides that, but Linux and the web build don't. With `--fix`,
files and folders that differ only in case are renamed to the name the data uses.

Checks: caveinfo files listed in data/resources/caveinfo_config.txt, unit files named by
caveinfo (`{f008}`), and map unit folders named by unit files (`# foldername`).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "data/assets"

# Files the data refers to that the romhack itself doesn't include.
KNOWN_MISSING = set()


def resolve(folder, name, fix, problems, what):
    """Finds `name` in `folder`, renaming a case-insensitive match when fixing."""
    exact = folder / name
    if exact.exists() and name in {p.name for p in folder.iterdir()}:
        return exact
    matches = [p for p in folder.iterdir()] if folder.is_dir() else []
    matches = [p for p in matches if p.name.lower() == name.lower()]
    if not matches:
        if exact.relative_to(ROOT).as_posix() not in KNOWN_MISSING:
            problems.append(f"missing {what}: {exact.relative_to(ROOT)}")
        return None
    if fix:
        tmp = matches[0].with_name(matches[0].name + ".case-tmp")
        matches[0].rename(tmp)
        tmp.rename(exact)
        print(f"renamed {matches[0].relative_to(ROOT)} -> {exact.name}")
        return exact
    problems.append(f"{what} case differs: data says {name}, file is {matches[0].relative_to(ROOT)}")
    return matches[0]


def main():
    args = sys.argv[1:]
    fix = "--fix" in args
    only = [a for a in args if not a.startswith("--")]
    problems = []

    caves = {}
    for line in (ROOT / "data/resources/caveinfo_config.txt").read_text(encoding="utf-8").splitlines():
        parts = [p.strip() for p in line.split(",")]
        if len(parts) >= 4:
            caves.setdefault(parts[0], []).append(parts[3])

    for game, files in sorted(caves.items()):
        game_dir = ASSETS / game
        if not game_dir.is_dir() or (only and game not in only):
            continue
        unitfiles, folders = set(), set()
        for name in files:
            path = resolve(game_dir / "caveinfo", name, fix, problems, "caveinfo file")
            if path:
                text = path.read_bytes().decode("shift_jis", errors="replace")
                unitfiles.update(re.findall(r"\{f008\}\s+-?\d+\s+(\S+)", text))
        for name in sorted(unitfiles):
            path = resolve(game_dir / "unitfiles", name, fix, problems, "unit file")
            if path:
                text = path.read_bytes().decode("shift_jis", errors="replace")
                folders.update(re.findall(r"^\s*(\S+)\s+#\s*foldername", text, re.M))
        for name in sorted(folders):
            resolve(game_dir / "mapunits", name, fix, problems, "map unit")
        print(f"{game}: {len(files)} caves, {len(unitfiles)} unit files, {len(folders)} map units")

    for p in problems:
        print(p)
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
