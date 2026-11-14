"""Builds data/resources/names.json: in-game English and Japanese names for treasures, teki
and caves, keyed by internal name.

    python tools/extract_names.py <mesRes_eng/pikmin2.bmg> <mesRes_jpn/pikmin2.bmg>

The .bmg files are the game's message tables, from `message/mesRes_eng.szs` and
`message/mesRes_jpn.szs` in an extracted PAL/multi-language Pikmin 2 disc (extract the SZS
archives with Wiimm's SZS Tools first). Message numbers follow the game's Piklopedia code
(pikmin2 decomp, plugProjectMorimuraU/zukan2D.cpp):

- Treasures: message 0100 + (position in otakara_config.txt) + 1. Exploration kit items
  (item_config.txt) continue after the last treasure. (Game::PelletList::Mgr::
  getOffsetFromDictionaryNo and TItemZukan's '0100_00' message set.)
- Teki: message 1300 + (Piklopedia position) + 1, in TEnemyZukan's order (ZUKAN_ORDER below).
- Caves: matched by English name, since the cave list here already has the English names.

A message number N is stored in the BMG's MID1 section as N << 8 (the low byte is a variant).
Objects with no Piklopedia entry (eggs, bomb rocks, hazards) get the names in EXTRA_TEKI,
which are not from the game's text.
"""

import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ZUKAN_ORDER = """Chappy YellowChappy BlueChappy Kochappy YellowKochappy BlueKochappy KumaChappy
KumaKochappy Baby FireChappy Catfish LeafChappy Tank Wtank Kabuto Rkabuto Mar Hanachirashi
MiniHoudai Kogane Wealthy Fart UjiA UjiB Tobi Armor Imomushi ElecBug TamagoMushi Jigumo Sarai
Demon BombSarai Fuefuki Kurage OniKurage FireOtakara ElecOtakara WaterOtakara GasOtakara
BombOtakara UmiMushiBlind Frog MaroFrog Tadpole BluePom RedPom YellowPom BlackPom WhitePom
RandPom Hana Sokkuri ShijimiChou Qurione Miulin PanModoki Pelplant HikariKinoko Clover Ooinu_l
Tanpopo Watage Tukushi Nekojarashi DaiodoRed Magaret Zenmai Wakame_l Queen SnakeCrow Damagumo
KingChappy OoPanModoki SnakeWhole Houdai UmiMushi BlackMan DangoMushi BigFoot BigTreasure""".split()

# Variants that share another teki's Piklopedia entry, with a suffix (ours, not the game's)
# to tell them apart where the game's name would be ambiguous.
SAME_AS = {
    "fkabuto": ("kabuto", "", ""),
    "fminihoudai": ("minihoudai", "", ""),
    "ooinu_s": ("ooinu_l", " (small)", "（小）"),
    "kareooinu_s": ("ooinu_l", " (small, withered)", "（小・枯れ）"),
    "kareooinu_l": ("ooinu_l", " (withered)", "（枯れ）"),
    "wakame_s": ("wakame_l", " (small)", "（小）"),
    "daiodogreen": ("daiodored", " (green)", "（緑）"),
}

# Spawnable objects with no Piklopedia entry. These names are ours, not the game's.
EXTRA_TEKI = {
    "egg": ("Egg", "タマゴ"),
    "bomb": ("Bomb-rock", "バクダン岩"),
    "hiba": ("Fire Geyser", "火の噴き出し口"),
    "gashiba": ("Gas Pipe", "毒ガスの噴き出し口"),
    "elechiba": ("Electrical Wire", "電線"),
    "rock": ("Falling Boulder", "落石"),
    "stone": ("Falling Boulder", "落石"),
}


def read_bmg(path, line_join):
    """Message number -> text, skipping control codes and joining lines with `line_join`."""
    b = Path(path).read_bytes()
    if b[:8] != b"MESGbmg1":
        sys.exit(f"{path} isn't a BMG file")
    sections, off = {}, 32
    for _ in range(struct.unpack(">I", b[12:16])[0]):
        size = struct.unpack(">I", b[off + 4 : off + 8])[0]
        sections[b[off : off + 4]] = b[off : off + size]
        off += size
    inf, dat, mid = sections[b"INF1"], sections[b"DAT1"][8:], sections[b"MID1"]
    count, entry_size = struct.unpack(">HH", inf[8:12])
    out = {}
    for i in range(count):
        start = struct.unpack(">I", inf[16 + i * entry_size : 20 + i * entry_size])[0]
        msg = struct.unpack(">I", mid[16 + i * 4 : 20 + i * 4])[0]
        raw, j = bytearray(), start
        while j < len(dat) and dat[j] != 0:
            if dat[j] == 0x1A:  # control code: 0x1A, total length, payload
                j += dat[j + 1]
                continue
            raw.append(dat[j])
            j += 1
        if msg & 0xFF == 0:
            text = raw.decode("shift_jis", errors="replace")
            lines = [" ".join(line.replace("　", " ").split()) for line in text.split("\n")]
            out[msg >> 8] = line_join.join(line for line in lines if line)
    return out


def config_names(path):
    text = path.read_bytes().decode("shift_jis")
    return re.findall(r"^\s*name\s+(\S+)", text, re.M)


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    # Japanese names are wrapped without spaces, English ones at spaces.
    en, ja = read_bmg(sys.argv[1], " "), read_bmg(sys.argv[2], "")

    def pair(msg):
        if msg not in en:
            sys.exit(f"message {msg:04} is missing")
        return {"en": en[msg], "ja": ja.get(msg, en[msg])}

    data = ROOT / "data/assets/pikmin2"
    treasures = config_names(data / "otakara_config.txt")
    items = config_names(data / "item_config.txt")
    names = {"treasures": {}, "teki": {}, "caves": {}}
    for i, name in enumerate(treasures + items):
        names["treasures"][name.lower()] = pair(100 + i + 1)

    for i, name in enumerate(ZUKAN_ORDER):
        names["teki"][name.lower()] = pair(1300 + i + 1)
    for name, (other, en_suffix, ja_suffix) in SAME_AS.items():
        base = names["teki"][other]
        names["teki"][name] = {"en": base["en"] + en_suffix, "ja": base["ja"] + ja_suffix}
    for name, (en_name, ja_name) in EXTRA_TEKI.items():
        names["teki"][name] = {"en": en_name, "ja": ja_name, "unofficial": True}

    by_english = {}
    for msg, text in en.items():
        by_english.setdefault(text.lower().removeprefix("the "), msg)
    for line in (ROOT / "data/resources/caveinfo_config.txt").read_text(encoding="utf-8").splitlines():
        parts = [p.strip() for p in line.split(",")]
        if len(parts) >= 4 and parts[0] == "pikmin2":
            msg = by_english.get(parts[1].lower())
            if msg is None:
                sys.exit(f"no message for cave {parts[1]}")
            names["caves"][parts[1]] = {"en": parts[1], "ja": ja[msg]}

    out = ROOT / "data/resources/names.json"
    out.write_text(json.dumps(names, ensure_ascii=False, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"{out}: {len(names['treasures'])} treasures, {len(names['teki'])} teki, {len(names['caves'])} caves")


if __name__ == "__main__":
    main()
