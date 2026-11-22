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
| Bikmin 269 | `bikmin` | DX definitive v1.1. Its main.dol differs from retail; generation is assumed to be vanilla's, and only the data has been checked. |

[docs/ADDING_ROMHACKS.md](docs/ADDING_ROMHACKS.md) explains how to add another romhack.

### Switch (Pikmin 2 HD)

The Switch release (2023) runs the same cave generation with a different `rand()`: musl libc's
64-bit generator instead of the GameCube's 32-bit one, so its seeds are 64 bits.

| | GameCube, Wii, Wii U | Switch |
|---|---|---|
| State update | `s * 0x41C64E6D + 0x3039` (mod 2^32) | `s * 6364136223846793005 + 1` (mod 2^64) |
| `rand()` | `(s >> 16) & 0x7FFF` | `s >> 33` |
| `randFloat()` | `rand() / 32768` | `rand() / 2^31` (as an f32) |
| Seeds | 8 hex digits | 16 hex digits |

The generator was worked out from gameplay video (Challenge Mode score rolls and the falling
letters of sublevel names), not from the Switch program, and no Switch layout has been compared
with the game yet.

- **Web page:** Advanced options → Seeds → Game version → Switch. The seed box then takes 16
  hex digits, and ‹ ›, Random seed, the judge and the seed search (with queries and counts) all
  use the Switch generator. Links carry it as `&rng=switch`.
- **Command line:** `--rng switch` for drawing layouts and `judge`.
- **Library:** `Layout::generate_with(Generator::Switch, seed, caveinfo)`, and
  `Query::matches_with` for queries.

On Switch, `(f32)rand()` can round up to 2^31, so about one `randInt` call in 2^25 would return
its limit and read past a list; CaveGen3 uses the last element instead. The aggregator, and the
command line's search, stats and count, are GameCube only for now.

## Web page

The page at https://jhawk4.github.io/CaveGen3/ runs the generator in the browser (compiled
to WebAssembly). Pick a sublevel and a seed, and it draws the layout with its treasures, teki
and gates. Links keep the sublevel and seed: `?sublevel=pikmin2:SCx-6&seed=1234ABCD`.
A link can also pick the judge, over the one saved from last time: `&judge=judge2` (any name in
the judge menu, or `judge=none`).

The page is in English and Japanese (日本語). It follows the browser's language; the button in the
corner switches, and `lang=ja` or `lang=en` in a link picks one. Treasures, teki and caves use
their in-game names in either language. Hover over or tap an icon on the map to see what it is.

**Advanced options** brings over much of the CaveGen GUI:

- **Objects**, **View**, **Generation details** and **Scores**: hide treasures (or only the
  fully buried ones, as CaveGen's `-drawNoBuriedItems`: depth at least its height), teki, plants,
  gates, gate HP (written on each gate) or exits; show only one landing or comedown view,
  the landing and comedown outlines, water, a grid and the treasure gauge; and draw spawn points (with Caveripper's markers: a duck for treasures, a leaf
  for plants, rings for easy teki), spawn order (CaveGen's `-drawSpawnOrder`), carry distances
  (`-drawPathDists`), the pod's facing (`-drawPodAngle`), facing directions, the game's unit,
  door, treasure and hole scores, teki scores, door links and numbers, waypoints and treasure
  carry paths.
- **QuickGlance** puts CaveGen's colored discs behind what matters (the pod, treasures, the exit,
  Gatling Groinks, Candypop Buds, Onions); **Pretty** draws the vanilla map units as CaveGen's
  pictures of the real floor instead of radar images. CaveGen took those as screenshots against a
  purple background with the game's spotlight, so `cargo run -p cavegen3-cli --example
  clean_pretty` cleans them before they're converted to WebP in `data/resources/pretty`: it
  removes the purple that shows through, evens out the lighting and each tile set's color, and
  fixes the dark band along each picture's edge, so neighboring tiles blend together.
  Toy and grassy caves have a floor backdrop under the pictures; `--example pretty_backdrops`
  cuts exactly one repeat of the floor texture out of CaveGen's framed backdrop pictures, so it
  tiles without seams, and it's drawn at a fixed size on the floor (40 pixels per map cell).
- **Colossal Caverns options** (shown only for Colossal Caverns): CaveGen's `-ultraRandomizer`,
  `-extraUnits` (Pikmin 251's units too) and `-5onion`. In links and on the command line they're
  suffixes on the name: `colossal+ultra+extra+5onion`. They're checked against CaveGen for 20
  seeds of each and of all three together.
- **Treasure gauge** shades the floor by how high the gauge's needle would be there: yellow
  while it reads below 0.3, orange from 0.3 and red from 0.8, deeper at each step of CaveGen's
  gauge marks. Per the game's code (`OtakaraSensor::calcLevel`, `Radar::Mgr::calcNearestTreasure`)
  the needle reads 0.05 + 0.95 (1 − d/900)² for the nearest treasure, with d the flat distance
  minus the treasure's bounding radius. Treasures carried by teki use the teki's position with no
  extra radius, since teki sizes aren't in the data.
- **Judge**: score the layout with any of CaveGen's judges (`score`, `pod`, `at`, `attk`,
  `cmat`, `key`, `colossal`, `mapunitcount`), or with `judge2`, a refit Pay off Debt judge
  (see [judge2](#judge2)) that also lists the treasures a runner likely skips.
- **Seed search**: test thousands of seeds (counting up from the seed, random, or game-RNG
  steps) and list the best by the judge, with each one's rank among the seeds tested, or the
  seeds matching a [Caveripper-style query](https://github.com/mayabyte/caveripper/blob/main/QUERY.md)
  such as `hole carry dist < 1000`, with the share that matched (Caveripper's `stats`). **Count**
  tallies how many of something each seed has (CaveGen's `-count`), e.g. `minihoudai` or
  `treasure`, and shows the distribution. Click a seed to open it; the search pauses while it
  draws.
- **CaveInfo**, below the map, laid out like Caveripper's caveinfo: the pod, exit and gates,
  the limits, a box per spawn group with each teki's minimum and weight (`x2 w10`), the
  treasures with their value and carry counts, and the map units' radar images.

Every option has a tooltip saying what it does.

The map pane keeps its size: the mouse wheel, a trackpad pinch, two fingers, the − / + buttons
or a double-click zoom about the pointer, zooming out shrinks the map against a dark background,
and dragging moves it anywhere, past the edges too. **Save image** saves the whole map.

**Landing outline** marks what's on screen when you land, and **Landing view only** shows just
that, turned as on screen. Each comes in three versions, since which way the camera faces depends
on whether you press L and which captain you play as:

- **(pod)**, white: after pressing L, which turns the camera behind the captain toward the cursor,
  the way the pod faces. Speedrunners mostly do this before the screen fades in.
- **(Olimar)**, red, and **(Louie)**, blue: without pressing L. Each floor starts both captains'
  cameras at `camera_angle` in `/user/Kando/aiConstants.txt` (290°, looking toward 110°), with
  the captains facing 0° (+Z), since caves have no map rotation (`setupFloatMemory`). The cave
  then makes the active captain the player, and `CameraMgr::changePlayerMode` centers the camera
  behind him only when that switches from Olimar. So as Olimar the camera looks toward 110° and
  as Louie toward 0°, on every floor, whichever way the pod faces (0° points down the map, 90°
  to the right). Checked against a speedrun
  that often doesn't press L: Louie on White Flower Garden 2 and Hole of Beasts 4, Olimar on
  Subterranean Complex 3 and Frontier Cavern 1, each within a few tens of degrees by eye.

In a link, `view=pod`, `view=olimar`, `view=louie` or `view=comedown` picks the view (the old
`landing=1` and `comedown=1` mean `view=pod`). The landing views use the game's "Far(Low)" camera settings
(`/user/Nishimura/Camera/caveCameraParms.txt`: 900 units back, 25° down, a 26° field of view),
looking the way the pod faces from just behind it, where the captain lands. The floor in view is
a trapezoid from about 330 units behind the pod to about 1,100 ahead, about 1,200 wide at the far
edge on a 4:3 screen (1,600 on 16:9; pick the screen in Advanced options). These numbers were
fitted to speedrun videos whose seeds were recovered from the video (Emergence Cave 1 and Hole of
Beasts 1, three runners): there the pod sits a little above the middle of the screen while the
Pikmin pour out. It's accurate to roughly a tenth of the screen, not to the pixel.

**Comedown outline** (yellow) marks what's on screen earlier, while the sublevel's name is up and
the pod comes down: the camera of the cutscene that starts each floor, `s0B_cv_coursein`
(`/user/Mukki/movie/s0B_cv_coursein/demo.szs`). Its first frame looks almost straight down from
1,321 units above the pod, with a 74° vertical field of view and the picture turned so the pod
faces down the screen, a little to the right. The view holds still until the name fades, then
the cutscene cuts to a low shot of the Pikmin and the pod lands. On the floor that's a rectangle
about 2,000 units tall and 2,660 wide on 4:3 (3,560 on 16:9), centered on the pod and turned
about 19° from its facing. Checked against Emergence Cave 1 in two runs: treasures in view land
within a pixel or two of where this camera puts them. The map grows to fit the outlines.
**Comedown view only** shows just that part of the floor, turned as it is on screen. Only one
of the four views can be on at a time.

The seed buttons can step through the game's RNG instead of counting by one (CaveGen's
`-consecutiveSeeds`); the seed search and the aggregator offer the same.

### Aggregator

`aggregator.html` recreates CaveGen's Aggregator: it generates many seeds of a sublevel, groups
the ones that look alike, and draws each group with how often each treasure, teki, gate and exit
spawned at each spot. Seeds can be grouped by the whole layout (CaveGen's default), the rooms
only, or the first room placed, as in CaveGen, or by **landing view**: what the camera shows when
you land: the map units in view (the default). Find the group that matches your landing to see where
the treasures probably are; a floor usually has a few dozen of these groups. Gray cells show how often other map
units were there.

Rotations don't split groups: the same arrangement turned any way is one group (CaveGen only
lined up the first room's rotation). Landing groups are turned so the camera faces up. Map
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

## Command line

`cavegen3` does everything the web page does from a terminal: layout and aggregator images as
PNG, and CaveInfo, judges, searches, stats and counts as text (or JSON). Build it once:

```
cargo build --release -p cavegen3-cli
```

Then run `target/release/cavegen3` (`cavegen3.exe` on Windows). With no command it draws layouts:

```
cavegen3 SCx-6 1234ABCD                      # one image: output/SCx6_1234ABCD.png
cavegen3 BK-4 -n 100 --seeds r -o bk4        # 100 random seeds into bk4/
cavegen3 SCx-6 1234ABCD --view pod           # what you see when you land, after pressing L
cavegen3 SCx-6 1234ABCD --view louie         # ... as Louie without L (or olimar)
cavegen3 SCx-6 1234ABCD --view comedown      # what you see under the sublevel's name
cavegen3 EC-1 63452B1B --show landingBox,landingBoxOlimar,landingBoxLouie,comedownBox,podAngle --screen 4:3
cavegen3 FC-3 0 --show spawnpoints,doorScores,quickGlance --hide water
cavegen3 251:AT-2 5                          # romhacks take a game prefix
cavegen3 colossal+ultra+5onion 0             # Colossal Caverns options
cavegen3 EC-1 306306F66F0443C4 --rng switch  # a Switch (Pikmin 2 HD) seed
```

| Command | What it does |
|---|---|
| `caves [game]` | Lists the caves and how to name their sublevels. |
| `caveinfo FC-3` | The floor's limits, teki (min, weight, group, what they carry), treasures (value, carry counts), gates, alcove teki and map units. `--json` for the raw data. |
| `judge BK-4 pod 1234ABCD [more seeds]` | Scores seeds with one of CaveGen's judges, or `judge2` (`--rng switch` for Switch seeds). |
| `judge SH-6 judge2 2B1980E4` | judge2 also prints the treasures likely skipped: `2B1980E4  70.84  likely skipped: bey_goma 100%`. |
| `search BK-4 --judge pod -n 100000 --top 10` | Tests seeds in parallel and lists the best, with each one's rank. `--query` keeps matching seeds and reports the share; `--count` adds a distribution; `--images DIR` draws the listed seeds; `--json`. |
| `stats SR-5 "BlackPom = 1"` | The share of seeds matching a query (Caveripper's `stats`). |
| `count SCx-7 minihoudai` | How many of something each seed has, as a distribution (CaveGen's `-count`). |
| `aggregate HoB-2 -n 20000` | CaveGen's aggregator: groups alike seeds and draws a map per group (`--group landing`, `layout`, `rooms` or `first`; `comedown` is the old name for `landing`), with what spawned at each spot and how often. |
| `options` | Lists the map options for `--show` and `--hide`. |

- Seeds are hex; leave the seed out for a random one. `--seeds c` counts up by one, `g` steps
  the game's RNG (CaveGen's `-consecutiveSeeds`), `r` picks random seeds; `--start` sets where
  `c` and `g` begin. Searches use every CPU core (about 60,000 judged layouts a second).
- `--show` and `--hide` take the web page's map options; treasures, teki, plants, gates, gate
  HP, exits and water are on by default. `--scale 2` doubles the size (1 is 40 pixels per grid
  cell); `--screen 4:3` sets the screen shape for the views and outlines. `--landing` (and its
  old name `--comedown`) is `--view pod`; `comedownBox` now means the comedown outline, and the
  pod's landing outline is `landingBox`.
- `--rng switch` uses the Switch release's generator (64-bit seeds; see
  [Switch](#switch-pikmin-2-hd)) for drawing and `judge`.
- It reads the repository's `data` folder; set `CAVEGEN3_DATA` or `--data` to use another.

### judge2

`judge2` (also `pod2`) is a Pay off Debt judge refit against timed Pay off Debt runs; it is not
part of CaveGen. CaveGen's `pod` judge assumes every counted treasure is collected. judge2 takes
the same parts (built by the `pod` code) for every choice of which counted treasures the runner
takes: all subsets when there are up to 4, otherwise taking all and skipping each one alone.
Each choice scores

```
1.22 walking + 0.78 carry + 0.60 hazard + 0.58 gate + 0.54 Glutton's Kitchen adjustments
  + 10 s per treasure taken + missing pokos / 2  [+ 100 s if SH-6's lens is taken and its carry route crosses water]
```

with no credit for bug pokos or optional treasures (runners take neither in practice). The
score is the choices' scores averaged with weights exp(−score / 5 s), so a treasure that isn't
worth its trip counts as likely skipped; the weights are also the chances of each choice. It
works wherever `pod` does and uses `pod`'s sublevel setup.

```
cavegen3 judge SH-6 judge2 2B1980E4 29698334
cavegen3 search SH-6 --judge judge2 -n 100000 --top 10
```

In the library, `JhawkJudge::new(JudgeType::Judge2, caveinfo)` scores with `judge`, and
`judge2(&layout)` returns the score with every choice and `skip_chances()`. The web page's judge
list has it too, and shows the treasures likely skipped (10% or more) under the score. It is
slower than `pod` by about the number of choices (one `pod`-like pass each, up to 16).

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
| 4. Rendering and the full CLI (generate, caveinfo, judge, search, stats, count, aggregate) | Done |
| 5. Web page | Done (layouts, map overlays, judges, seed search, CaveInfo, aggregator) |
| 6. Discord bot | Planned |

## Layout

| Path | What it is |
|---|---|
| `crates/cavegen3` | The core library. |
| `crates/cavegen3-web` | The library compiled to WebAssembly for the web page. |
| `crates/cavegen3-cli` | The `cavegen3` command, which saves layouts as PNG images. |
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
- `crates/cavegen3/tests/judge2.rs`: judge2 scores and skip chances for fixed seeds, from the
  program it was fitted with (16,308 layouts of the 54 Pay off Debt sublevels matched exactly).
- `crates/cavegen3/tests/switch.rs`: every vanilla sublevel generates 200 Switch layouts without
  error. `rng.rs`'s tests check the Switch stream against musl's formula, `advance`, and the f32
  rounding of `randFloat` and `randInt`.
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
