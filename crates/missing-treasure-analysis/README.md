# missing-treasure-analysis

Studies of cave maps that close before all their rooms are placed. Such a map can be short of a
treasure, or of a boss, as in [Shower Room 7's missing boss](../../docs/SR7_MISSING_BOSS.md).

The crate covers:
- how likely that is for each sublevel;
- fast hunts for seeds that do it;
- steering generation by choosing what `rand()` returns.

It uses CaveGen3's generator one placement step at a time, through the `cavegen3` crate's
`internals` feature (`generate::internals::MapGen`, which also turns on the `tape` feature).
Normal `cavegen3` builds leave both features off, so this code adds nothing to them.

## Library

| Module | What it has |
| --- | --- |
| `closure` | `closes_with_rooms` (stops at the first extra room), and `ClosureFilter`, which rules out most seeds before any generation. |
| `splitting` | `missing_treasure_odds` and `room_failure_layouts`: multilevel splitting estimates that extend `cavegen3::generate::room_failure_odds`. |
| `steer` | `run_tape`, `layout_from_tape` and a beam `search` for `rand()` tapes that give a closed map short of a treasure. |

### ClosureFilter

`ClosureFilter` is for maps that close with only the start room.

**Step 1.** The first placement step's corridor roll always falls on the same `rand()` call:
- call 47 on WFG-4, after the unit list shuffles, the teki slots and the start room;
- one LCG jump, a single multiply and add, reaches it.

When that roll says "room first" and a room fits at every door of every possible start room, a
second room is certain and the seed is ruled out.

**Step 2.** For the seeds left, the filter replays the first hallway placement and the second
step's roll on small arrays. The map geometry comes from tables that the real generator builds
once. On WFG-4 (Switch), the two steps together leave 4% of seeds for real generation:

| Method, WFG-4 Switch, one thread | Seeds/s |
| --- | --- |
| Place all rooms (`cavegen3::generate::rooms_placed`) | ~110–140k |
| Stop at the 2nd room | ~190–220k |
| Step-1 filter, then generate | ~600k |
| Step-1+2 filter, then generate | ~1.1–1.5M |

`hunt_bench` checks the filter on every seed it times: each seed it rules out really places a
second room within the steps it looked at.

| Sublevel | Left for generation |
| --- | --- |
| WFG-4 Switch | 4.0% |
| WFG-4 GC | 4.1% |
| SH-4 | 1.3% |
| SR-5 | 13% |
| SR-7 | 89% |

SR-7 gains little from it.

## Examples

`cargo run --release -p missing-treasure-analysis --example NAME -- ARGS`:

| Example | What it does |
| --- | --- |
| `missing_odds [gc\|switch] PARTICLES RUNS SUBLEVEL...` | Chance per seed (ideal RNG) that the map closes with only its start room and is short of a treasure. |
| `closure_hunt SUBLEVEL [gc\|switch] [how many] [first batch] [threads]` | Hunts random seeds for such maps. It works in batches of 2^28 seeds; give each machine its own batch range. |
| `hunt_bench SUBLEVEL [gc\|switch] [seeds]` | Single-thread timing of the hunt's stages, and a check of `ClosureFilter` against the generator. |
| `closure_treasures [particles] [runs] [story\|challenge\|all] [sublevel...]` | For each sublevel with treasures, the chance its map closes short of rooms, and how often such a map is also short of a treasure. |
| `treasure_slack [seeds] [story\|challenge\|all]` | How close each sublevel comes to running out of treasure spots, over random GC seeds. |
| `start_spots` | Story sublevels where a map with only its start room would be short of treasure spots. |
| `treasure_spots SUBLEVEL...` | The treasure spots each unit of a sublevel offers. |
| `steer [gc\|switch] [beam] [tries] [restarts] SUBLEVEL...` | Chooses `rand()` values that make a map close short of rooms and a treasure, then measures how many bits of seed that takes. |
| `seed_bits SR-7 31D70855` | How tightly a real seed's `rand()` values are pinned by its map closing short of rooms. |
| `draw_tape SUBLEVEL gc\|switch TAPE OUT.png` | Draws the map a tape generates. |

## Results so far

From `missing_odds` with an ideal random number generator, over 32 runs each:

| Sublevel | Closes with one room and is short of a treasure | Runs that reached closing |
| --- | --- | --- |
| SR-5 | 3.6e-14 ± 0.8e-14 (1 in 2.8e13) | 32 of 32 |
| WFG-4 | 6.7e-14 ± 3.4e-14 (1 in 1.5e13) | 5 of 32 |
| SH-4 | 3.2e-19 ± 2.0e-19 (1 in 3.2e18) | 3 of 32 |

In every run, every closed one-room map was short of a treasure.
