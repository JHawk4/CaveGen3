//! Steering generation by choosing what `rand()` returns: for finding which values a seed would
//! need to give a rare map (one that closes before all its rooms are placed and is short of a
//! treasure), and how tightly each value is constrained.

use std::hash::{Hash, Hasher};

use cavegen3::{
    Layout,
    caveinfo::{CaveInfo, RoomType},
    generate::internals::MapGen,
    rng::Rng,
};

use crate::{splitmix, splitting::treasures};

/// What a tape of `rand()` values generates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub rooms: u32,
    pub num_rooms: u32,
    pub treasures: u32,
    pub max_treasures: u32,
    /// How many `rand()` calls happened up to and including treasure placement.
    pub calls: usize,
}

impl Outcome {
    /// The map closed before all its rooms were placed and is short of a treasure.
    pub fn short_and_missing(&self) -> bool {
        self.rooms < self.num_rooms && self.treasures < self.max_treasures
    }
}

/// Generates up to treasure placement with `rand()` playing `values` (fresh values after them).
pub fn run_tape(caveinfo: &CaveInfo, generator: cavegen3::Generator, values: &[u32]) -> Outcome {
    let mut g = MapGen::new(Rng::tape(generator, values.to_vec(), 0x7A9E), caveinfo);
    g.place_map_units();
    let rooms = g.rooms_placed();
    g.finish_through_items();
    outcome(&g, rooms)
}

fn outcome(g: &MapGen, rooms: u32) -> Outcome {
    let ci = g.caveinfo();
    Outcome {
        rooms,
        num_rooms: ci.num_rooms,
        treasures: treasures(g),
        max_treasures: ci.max_treasures,
        calls: g.rng().tape_ref().map_or(0, |t| t.pos),
    }
}

fn item_alcoves(g: &MapGen) -> usize {
    g.units()
        .iter()
        .filter(|u| u.unit.room_type == RoomType::DeadEnd && u.unit.unit_folder_name.contains("item"))
        .count()
}

fn map_hash(g: &MapGen) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for u in g.units() {
        (u.unit.unit_folder_name.as_str(), u.unit.rotation, u.x, u.z).hash(&mut h);
    }
    h.finish()
}

/// Statistics of a [`search`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SearchStats {
    pub restarts: u64,
    /// Maps that closed short of rooms, and how many of those were also short of a treasure.
    pub closures: u64,
    pub missing: u64,
}

/// Beam search over placement steps, with every `rand()` value recorded on a tape: each step,
/// every kept map tries `tries` random continuations, and the `beam` maps with the fewest rooms
/// (then fewest item alcoves, open doors a room fits at and open doors) are kept. A map that
/// closes short of rooms is finished through treasure placement; the first that is also short
/// of a treasure is returned as its tape. Gives up after `restarts` fresh starts.
pub fn search(
    caveinfo: &CaveInfo,
    generator: cavegen3::Generator,
    beam: usize,
    tries: usize,
    restarts: u64,
    seed: u64,
) -> (Option<Vec<u32>>, SearchStats) {
    let mut mix = seed;
    let mut next = move || splitmix(&mut mix);
    let mut stats = SearchStats::default();
    let mut attempts = 0;
    while stats.restarts < restarts && attempts < restarts * 1000 {
        attempts += 1;
        stats.restarts += 1;
        let mut g = MapGen::new(Rng::tape(generator, Vec::new(), next()), caveinfo);
        if !g.place_start() {
            return (None, stats);
        }
        // Only a start room with too few treasure spots can leave a one-room map short.
        let start_spots = g.units()[0].unit.spawnpoints.iter().filter(|s| s.group == 2).count() as u32;
        if start_spots >= caveinfo.max_treasures
            && caveinfo.cave_units.iter().any(|u| {
                u.room_type == RoomType::Room
                    && u.spawnpoints.iter().any(|s| s.group == 7)
                    && (u.spawnpoints.iter().filter(|s| s.group == 2).count() as u32) < caveinfo.max_treasures
            })
        {
            stats.restarts -= 1;
            continue;
        }
        let mut alive = vec![g];
        for steps in 1..=500 {
            let mut children: Vec<((usize, usize, usize, usize, u64), MapGen)> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for parent in &alive {
                for _ in 0..tries {
                    let mut c = parent.clone();
                    c.rng_mut().reseed_fresh(next());
                    let open = c.placement_step(steps);
                    if c.rooms_placed() >= caveinfo.num_rooms {
                        continue;
                    }
                    if !open {
                        stats.closures += 1;
                        let rooms = c.rooms_placed();
                        c.finish_through_items();
                        if outcome(&c, rooms).short_and_missing() {
                            stats.missing += 1;
                            let t = c.rng().tape_ref().unwrap();
                            return (Some(t.values[..t.pos].to_vec()), stats);
                        }
                        continue;
                    }
                    if !seen.insert(map_hash(&c)) {
                        continue;
                    }
                    let key = (c.rooms_placed() as usize, item_alcoves(&c), c.roomable_doors(), c.open_doors().len(), next());
                    children.push((key, c));
                }
            }
            if children.is_empty() {
                eprintln!("{} restart {}: stuck after {steps} steps ({} closures so far)", caveinfo.name(), stats.restarts, stats.closures);
                break;
            }
            children.sort_by_key(|c| c.0);
            children.truncate(beam);
            alive = children.into_iter().map(|c| c.1).collect();
        }
    }
    (None, stats)
}

/// The full layout a tape of `rand()` values generates (for drawing steered maps).
pub fn layout_from_tape<'a>(caveinfo: &'a CaveInfo, generator: cavegen3::Generator, values: &[u32]) -> Layout<'a> {
    let mut g = MapGen::new(Rng::tape(generator, values.to_vec(), 0x7A9E), caveinfo);
    g.place_map_units();
    g.finish();
    g.into_layout()
}
