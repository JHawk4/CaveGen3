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
| Pikmin 2: 216, Bikmin | | Listed, but their files aren't included yet. |

[docs/ADDING_ROMHACKS.md](docs/ADDING_ROMHACKS.md) explains how to add another romhack.

## Web page

The page at https://jhawk4.github.io/CaveGen3/ runs the generator in the browser (compiled
to WebAssembly). Pick a sublevel and a seed, and it draws the layout with its treasures, teki
and gates. Links keep the sublevel and seed: `?sublevel=pikmin2:SCx-6&seed=1234ABCD`.

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

## Status

| Milestone | State |
|---|---|
| 1. Repo, bundled data, RNG and math, Caveripper as a test reference | Done |
| 2. New generator matching CaveGen and Caveripper | Done |
| 3. Waypoints, judge, queries and parallel search (library) | Done |
| 4. Rendering and the full CLI (generate, caveinfo, search, search-from, search-special, stats, filter, judge) | Next |
| 5. Web page | Done (generate and view layouts) |
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
