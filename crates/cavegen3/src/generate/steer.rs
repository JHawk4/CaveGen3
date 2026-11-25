//! Steering generation by choosing what `rand()` returns: for finding which values a seed would
//! need to give a rare map (one that closes before all its rooms are placed and is short of a
//! treasure), and how tightly each value is constrained.

use std::hash::{Hash, Hasher};

use super::*;

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
pub fn run_tape(caveinfo: &CaveInfo, generator: crate::Generator, values: &[u32]) -> Outcome {
    let mut g = Generator::new(Rng::tape(generator, values.to_vec(), 0x7A9E), caveinfo);
    g.sort_units_and_allocate_teki();
    g.place_map_units();
    let rooms = g.rooms_placed;
    g.finish_through_items();
    outcome(&g, rooms)
}

fn outcome(g: &Generator, rooms: u32) -> Outcome {
    let treasures = g
        .spawnpoints
        .iter()
        .map(|sp| sp.contains.iter().filter(|so| matches!(so, SpawnObject::Item(_))).count() as u32)
        .sum();
    Outcome {
        rooms,
        num_rooms: g.ci.num_rooms,
        treasures,
        max_treasures: g.ci.max_treasures,
        calls: g.rng.tape_ref().map_or(0, |t| t.pos),
    }
}

impl Generator<'_> {
    /// [`Generator::finish`] up to and including treasure placement.
    fn finish_through_items(&mut self) {
        self.create_spawnpoints_and_recenter();
        self.place_ship();
        self.set_score();
        if !self.ci.is_final_floor {
            self.place_hole(SpawnObject::Hole(self.ci.exit_plugged));
        }
        if self.ci.is_final_floor || self.ci.has_geyser {
            self.place_hole(SpawnObject::Geyser(self.ci.is_challenge_mode() && self.ci.is_final_floor));
        }
        self.place_seam_teki();
        self.place_special_teki();
        self.place_hard_teki();
        self.place_easy_teki();
        self.set_score();
        self.place_plants();
        self.place_items();
    }

    fn item_alcoves(&self) -> usize {
        self.units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::DeadEnd && u.unit.unit_folder_name.contains("item"))
            .count()
    }

    fn map_hash(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for u in &self.units {
            (u.unit.unit_folder_name.as_str(), u.unit.rotation, u.x, u.z).hash(&mut h);
        }
        h.finish()
    }
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
/// (then fewest open doors a room fits at, open doors and item alcoves) are kept. A map that
/// closes short of rooms is finished through treasure placement; the first that is also short
/// of a treasure is returned as its tape. Gives up after `restarts` fresh starts.
pub fn search(
    caveinfo: &CaveInfo,
    generator: crate::Generator,
    beam: usize,
    tries: usize,
    restarts: u64,
    seed: u64,
) -> (Option<Vec<u32>>, SearchStats) {
    let mut mix = seed;
    let mut next = move || {
        mix = mix.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = mix;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    let mut stats = SearchStats::default();
    let mut attempts = 0;
    while stats.restarts < restarts && attempts < restarts * 1000 {
        attempts += 1;
        stats.restarts += 1;
        let mut g = Generator::new(Rng::tape(generator, Vec::new(), next()), caveinfo);
        g.sort_units_and_allocate_teki();
        if !g.place_start() {
            return (None, stats);
        }
        // Only a start room with too few treasure spots can leave a one-room map short.
        let start_spots = g.units[0].unit.spawnpoints.iter().filter(|s| s.group == 2).count() as u32;
        if start_spots >= g.ci.max_treasures && g.ci.cave_units.iter().any(|u| {
            u.room_type == RoomType::Room
                && u.spawnpoints.iter().any(|s| s.group == 7)
                && (u.spawnpoints.iter().filter(|s| s.group == 2).count() as u32) < g.ci.max_treasures
        }) {
            stats.restarts -= 1;
            continue;
        }
        let mut alive = vec![g];
        for steps in 1..=500 {
            let mut children: Vec<((usize, usize, usize, usize, u64), Generator)> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for parent in &alive {
                for _ in 0..tries {
                    let mut c = parent.clone();
                    c.rng.reseed_fresh(next());
                    let open = c.placement_step(steps);
                    if c.rooms_placed >= c.ci.num_rooms {
                        continue;
                    }
                    if !open {
                        stats.closures += 1;
                        let rooms = c.rooms_placed;
                        c.finish_through_items();
                        if outcome(&c, rooms).short_and_missing() {
                            stats.missing += 1;
                            let t = c.rng.tape_ref().unwrap();
                            return (Some(t.values[..t.pos].to_vec()), stats);
                        }
                        continue;
                    }
                    if !seen.insert(c.map_hash()) {
                        continue;
                    }
                    let key = (c.rooms_placed as usize, c.item_alcoves(), c.roomable_doors(), c.open.len(), next());
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
