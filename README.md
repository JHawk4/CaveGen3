# CaveGen3

Fast Pikmin 2 cave generation, seed search and layout judging.

**Try it in your browser: https://jhawk4.github.io/CaveGen3/**

CaveGen3 aims to do everything [Caveripper](https://github.com/mayabyte/caveripper) does except
extracting game files, with a rewritten, much faster generator. It's checked against JHawk's
[CaveGen](https://github.com/JHawk4/CaveGen) and against Caveripper itself on every change.
The game data it needs is included, so there's nothing to extract.

## Supported games

| Game | Folder | Notes |
|---|---|---|
| Pikmin 2 | `pikmin2` | Story and challenge mode. |
| Pikmin 251 | `251` | |
| Pikmin 2: New Year | `newyear` | Timeless Realm (CH28) floor 3 is missing a unit file in the hack's own data. |
| Colossal Caverns | `colossal` | |
| Pikmin 216 | `216` | Version 1.3. Its generation code is assumed to be vanilla's; only the data has been checked. |
| Bikmin 269 | `bikmin` | Listed, but its files aren't included yet: it ships only as a patch to the retail ISO. |

[docs/ADDING_ROMHACKS.md](docs/ADDING_ROMHACKS.md) explains how to add another romhack.

## Web page

The page at https://jhawk4.github.io/CaveGen3/ runs the generator in the browser (compiled
to WebAssembly). Pick a sublevel and a seed, and it draws the layout with its treasures, teki
and gates. Links keep the sublevel and seed: `?sublevel=pikmin2:SCx-6&seed=1234ABCD`.

The page is in English and Japanese (日本語). It follows the browser's language; the button in the
corner switches, and `lang=ja` or `lang=en` in a link picks one. Treasures, teki and caves use
their in-game names in either language. Hover over or tap an icon on the map to see what it is.

**Advanced options** brings over much of the CaveGen GUI:

- **Show on the map**: hide treasures, teki, plants, gates or exits, and draw spawn points,
  facing directions, the game's unit, door, treasure and hole scores, teki scores, door links
  and numbers, waypoints, treasure carry paths, treasure gauge rings and a grid.
- **Judge**: score the layout with any of CaveGen's judges (`score`, `pod`, `at`, `attk`,
  `cmat`, `key`, `colossal`, `mapunitcount`).
- **Seed search**: test thousands of seeds (counting up from the seed, or random) and list the
  best by the judge, with each one's rank among the seeds tested, or the seeds matching a
  [Caveripper-style query](https://github.com/mayabyte/caveripper/blob/main/QUERY.md) such as
  `hole carry dist < 1000`. Click a seed to open it.
- **CaveInfo**, below the map: the sublevel's limits and the teki, treasures, gates, alcove teki
  and map units it generates from, with their weights.

The map pane keeps its size: the mouse wheel, a trackpad pinch, two fingers, the − / + buttons
or a double-click zoom about the pointer, zooming out shrinks the map against a dark background,
and dragging moves it anywhere, past the edges too. **Save image** saves the whole map.

**Comedown only** shows just what's on screen when you land, turned so the camera faces up as
in the game (`comedown=1` in a link turns it on), and **Comedown outline** marks it on the full
map. It's worked out from the landing camera in the game's files
(`/user/Nishimura/Camera/caveCameraParms.txt`, the "Mid(Low)" settings `PlayCamera::init`
starts with: 600 units back, 21° down, a 23° field of view). The camera looks the way the pod
faces, so the floor in view is a trapezoid from a little behind the pod to about 1,000 units
ahead, about 1,100 wide at the far edge on a 16:9 screen (800 on 4:3; pick the screen in
Advanced options). This was checked against a run video (Hole of Beasts 2, seed `23A06B6B`),
which also set how far ahead of the pod the camera aims when you get control.

The seed buttons can step through the game's RNG instead of counting by one (CaveGen's
`-consecutiveSeeds`); the seed search and the aggregator offer the same.

### Aggregator

`aggregator.html` recreates CaveGen's Aggregator: it generates many seeds of a sublevel, groups
the ones that look alike, and draws each group with how often each treasure, teki, gate and exit
spawned at each spot. Seeds can be grouped by the whole layout (CaveGen's default), the rooms
only, or the first room placed, as in CaveGen, or by **comedown**: what the camera shows when you
land, either the map units alone or the map units and the objects in view. Find the group that
matches your landing to see where the treasures probably are. Gray cells show how often other map
units were there.

Rotations don't split groups: the same arrangement turned any way is one group (CaveGen only
lined up the first room's rotation). Comedown groups are turned so the camera faces up. Map
units that look the same after a quarter or half turn count as the same in any of those
rotations: the aggregator checks each unit's shape, doors and water, and the page checks its
radar image. In vanilla Pikmin 2 these include the four-way junctions, the cross rooms (like
Frontier Cavern's four-way room) and straight hallways. Their spawn points don't always line up
after a turn, so a group with such a room overlays the spawn spots of each way it was turned.

### Suggestion box

`suggest.html` sends suggestions to the same Google Form as the Pikmin WR mirror (the form's
fields are in that page). Its hidden "page" field is `cavegen3/<lang>/<page the visitor came
from>`, so CaveGen3's entries can be told apart in the shared responses sheet.

To embed it in another site, add `embed=1` for a compact version with just the controls and
the map:

```html
<iframe src="https://jhawk4.github.io/CaveGen3/?embed=1&sublevel=pikmin2:SCx-6&seed=1234ABCD"
        width="800" height="700" style="border:0"></iframe>
```

To build it locally:

```
rustup target add wasm32-unknown-unknown
cargo build --release -p cavegen3-web --target wasm32-unknown-unknown
python tools/build_site.py
python -m http.server --directory site
```

GitHub Actions publishes it on every push to `main` (`.github/workflows/pages.yml`).

## In-game names

`data/resources/names.json` maps internal names to the names the game shows, in English and
Japanese: every treasure and exploration kit item (`ahiru` → Rubber Ugly / 天下一ブサイク),
every Piklopedia teki (`Kochappy` → Dwarf Red Bulborb / アカコチャッピー), and the Pikmin 2
caves. It was built by `tools/extract_names.py` from the game's own message tables
(`message/mesRes_eng.szs` and `mesRes_jpn.szs`), using the message numbering in the game's
Piklopedia code. Objects with no Piklopedia entry (eggs, bomb-rocks, gas pipes and the like),
and the "(small)", "(withered)" and "(green)" variant labels, are our wording and marked
`"unofficial"` where they stand alone. Romhacks reuse the vanilla names; their own new
treasures and teki show their internal names.

## Status

| Milestone | State |
|---|---|
| 1. Repo, bundled data, RNG and math, Caveripper as a test reference | Done |
| 2. New generator matching CaveGen and Caveripper | Done |
| 3. Waypoints, judge, queries and parallel search (library) | Done |
| 4. Rendering and the full CLI (generate, caveinfo, search, search-from, search-special, stats, filter, judge) | Next |
| 5. Web page | Done (layouts, map overlays, judges, seed search, CaveInfo, aggregator) |
| 6. Discord bot | Planned |

## Layout

| Path | What it is |
|---|---|
| `crates/cavegen3` | The core library. |
| `crates/cavegen3-web` | The library compiled to WebAssembly for the web page. |
| `crates/caveripper-ref` | Caveripper's library with our fixes, kept only as a test reference. |
| `web/` | The web page. |
| `data/assets/<game>` | Game data, one folder per game. |
| `data/resources` | The cave list, fonts, icons and judge tables. |
| `tools/` | Scripts for importing romhack data, checking it and building the web page. |
| `testdata/` | Expected output from CaveGen, used by the tests. |
| `reference/` | Values from the game's RNG and square root, used by the tests. |

## Building and testing

Needs stable Rust (the toolchain is pinned in `rust-toolchain.toml`).

```
cargo test --release
```

What the tests check:

- `crates/cavegen3/tests/cavegen_expect.rs`: CaveGen3's layouts match CaveGen's output for
  16,400 vanilla layouts, 100 Colossal Caverns layouts and about 8,000 Pikmin 251 and New Year
  layouts. A handful of romhack sublevels where CaveGen and Caveripper disagree are listed
  there and skipped; CaveGen3 follows Caveripper on them.
- `crates/cavegen3/tests/vs_reference.rs`: CaveGen3 and Caveripper produce identical layouts
  (units, scores, door scores, and every object's exact position) for 600 seeds of every
  floor of every game. Set `VS_REFERENCE_SEEDS` for more.
- `crates/cavegen3/tests/parse_vs_reference.rs`: both parse every included cave identically.
- `crates/cavegen3/tests/waypoints_vs_reference.rs`: waypoint graphs and carry paths match.
- `crates/cavegen3/tests/query_vs_reference.rs`: queries give the same answers on every seed tested.
- `crates/cavegen3/tests/jhawk_judge_vs_cavegen.rs`: 72,500 judge scores match CaveGen.
- `crates/caveripper-ref/tests`: the reference passes CaveGen's expect tests and 72,500
  judge scores.

`cargo run --release -p cavegen3 --example bench` times both generators on every floor.

## Credits and license

CaveGen3's code is MIT-licensed (see `LICENSE`). Parts of it are ported from Caveripper by
mayabyte and contributors, also MIT; see `THIRD_PARTY_NOTICES.md`. Generation behavior follows
JHawk's CaveGen and the [pikmin2 decompilation](https://github.com/projectPiki/pikmin2).

The game data in `data/assets` is not covered by that license. Pikmin 2 is © Nintendo; its files
are included so the tools work without extracting a copy of the game. Romhack files belong to
their authors and are included with their permission. The fonts in `data/resources` are under
the SIL Open Font License, and the special icons there come from Caveripper's repository.
