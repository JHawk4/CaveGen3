# CaveGen3

Fast Pikmin 2 cave generation, seed search and layout judging.

CaveGen3 aims to do everything [Caveripper](https://github.com/mayabyte/caveripper) does except
extracting game files, with a rewritten, much faster generator. It's checked against JHawk's
CaveGen and against Caveripper itself on every change. The game data it needs is bundled in this
repository.

> **Keep this repository private.** `data/` contains files from Pikmin 2 and its romhacks.
> They belong to their owners (Nintendo for the base game) and are not covered by this
> project's MIT license.

## Status

| Milestone | State |
|---|---|
| 1. Repo, bundled data, RNG and math, Caveripper as a test reference | Done |
| 2. New generator matching CaveGen and Caveripper | Done |
| 3. Waypoints, judge, queries, search / stats / filter / search-from / search-special | Planned |
| 4. Rendering and the full CLI | Planned |
| 5. Web build | Planned |
| 6. Discord bot | Planned |

## Layout

| Path | What it is |
|---|---|
| `crates/cavegen3` | The core library. |
| `crates/caveripper-ref` | Caveripper's library with our fixes, kept only as a test reference. |
| `data/assets/<game>` | Game data: `pikmin2` and `colossal` so far. |
| `data/resources` | Fonts, icons and tables (cave list, judge tables). |
| `testdata/` | Expected output from CaveGen, used by the tests. |
| `reference/` | Values from the game's RNG and square root, used by the tests. |

## Building and testing

Needs stable Rust (the toolchain is pinned in `rust-toolchain.toml`).

```
cargo test
```

What the tests check:

- `crates/cavegen3/tests/cavegen_expect.rs`: CaveGen3's layouts match CaveGen's output for
  16,400 vanilla layouts and 100 Colossal Caverns layouts.
- `crates/cavegen3/tests/vs_reference.rs`: CaveGen3 and Caveripper produce identical layouts
  (units, scores, door scores, and every object's exact position) for 600 seeds of every
  floor. Set `VS_REFERENCE_SEEDS` for more.
- `crates/cavegen3/tests/parse_vs_reference.rs`: both parse every bundled cave identically.
- `crates/caveripper-ref/tests`: the reference passes CaveGen's expect tests and 72,500
  judge scores.

`cargo run --release -p cavegen3 --example bench` times both generators on every floor.

## Credits and license

CaveGen3 is MIT-licensed (see `LICENSE`). Parts of it are ported from Caveripper by mayabyte
and contributors, also MIT; see `THIRD_PARTY_NOTICES.md`. Generation behavior follows JHawk's
CaveGen and the [pikmin2 decompilation](https://github.com/projectPiki/pikmin2).
