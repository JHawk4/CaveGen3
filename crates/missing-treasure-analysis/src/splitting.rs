//! Multilevel splitting estimates of how often a map closes short of rooms, and with what in it.
//!
//! These extend `cavegen3::generate::room_failure_odds` (which stays as it is): finer levels,
//! levels on how many ways a room still fits rather than on doors, a condition on the start room,
//! a cap on rooms placed, and the closed maps themselves.

use cavegen3::{Layout, SpawnObject, caveinfo::CaveInfo, generate::internals::MapGen, rng::Rng};

use crate::splitmix;

/// Levels for sublevels with more rooms and doors: at most 12 down to 0 doors where a room fits,
/// then no such door and at most 10 down to 1 open doors, then closed.
fn fine_failure_levels() -> Vec<(usize, usize)> {
    let mut levels: Vec<(usize, usize)> = (0..=12).rev().map(|r| (r, usize::MAX)).collect();
    levels.extend((1..=10).rev().map(|o| (0, o)));
    levels.push((0, 0));
    levels
}

/// Levels on how many ways a room still fits (open door, room unit, unit door), then on open doors.
fn fit_levels() -> Vec<(usize, usize)> {
    let mut levels: Vec<(usize, usize)> = [64, 48, 36, 28, 22, 18, 15, 12, 10, 8, 7, 6, 5, 4, 3, 2, 1, 0].iter().map(|&r| (r, usize::MAX)).collect();
    levels.extend((1..=10).rev().map(|o| (0, o)));
    levels.push((0, 0));
    levels
}

/// Like `room_failure_odds` with finer levels, also returning the maps that closed short of rooms
/// at the last level, each finished into a full layout (teki, treasures, gates) with a fresh
/// random state.
pub fn room_failure_layouts(
    caveinfo: &CaveInfo,
    generator: cavegen3::Generator,
    particles: usize,
    seed: u64,
) -> (f64, Vec<f64>, Vec<Layout<'_>>) {
    let (odds, shares, closed) = room_failure_run(caveinfo, generator, particles, seed, fine_failure_levels(), u32::MAX, &|_| true, false);
    let mut mix = seed ^ 0xF1A1_5EED;
    let layouts = closed
        .into_iter()
        .map(|mut g| {
            mix = mix.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *g.rng_mut() = Rng::with(generator, mix);
            g.finish();
            g.into_layout()
        })
        .collect();
    (odds, shares, layouts)
}

/// Estimates the chance a seed's map closes with only its start room and is short of a
/// treasure: splitting where only maps whose start room has fewer treasure spots than treasures
/// take part, maps that place a second room fail, and the levels count the ways a room still
/// fits. The closed maps are then finished with fresh random states, and the share short of a
/// treasure scales the closing odds. Returns (chance, closing odds, level shares, closed maps,
/// short maps); the first share is the start room's.
pub fn missing_treasure_odds(
    caveinfo: &CaveInfo,
    generator: cavegen3::Generator,
    particles: usize,
    seed: u64,
) -> (f64, f64, Vec<f64>, usize, usize) {
    let start_ok = |g: &MapGen| (g.units()[0].unit.spawnpoints.iter().filter(|s| s.group == 2).count() as u32) < caveinfo.max_treasures;
    let (odds, shares, closed) = room_failure_run(caveinfo, generator, particles, seed, fit_levels(), 1, &start_ok, true);
    let mut mix = seed ^ 0x5407_7EED;
    let mut short = 0;
    for mut g in closed.iter().cloned() {
        mix = mix.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *g.rng_mut() = Rng::with(generator, mix);
        g.finish_through_items();
        if g.rooms_placed() < caveinfo.num_rooms && treasures(&g) < caveinfo.max_treasures {
            short += 1;
        }
    }
    let p = if closed.is_empty() { 0.0 } else { odds * short as f64 / closed.len() as f64 };
    (p, odds, shares, closed.len(), short)
}

/// Treasures placed so far.
pub(crate) fn treasures(g: &MapGen) -> u32 {
    g.spawnpoints()
        .iter()
        .map(|sp| sp.contains.iter().filter(|so| matches!(so, SpawnObject::Item(_))).count() as u32)
        .sum()
}

/// The splitting loop. Each level is (at most this many doors where a room fits, or ways a room
/// fits if `count_fits`; at most this many open doors), with (0, 0) meaning closed. Maps grow
/// until they reach the next level, or fail by placing their last room, placing more than
/// `max_rooms`, or reaching the game's 500 steps. Maps whose start fails `start_ok` fail at once,
/// and that share is the first factor. Returns the odds, the shares (the start's first) and the
/// maps that reached the last level.
#[allow(clippy::too_many_arguments)]
fn room_failure_run<'a>(
    caveinfo: &'a CaveInfo,
    generator: cavegen3::Generator,
    particles: usize,
    seed: u64,
    levels: Vec<(usize, usize)>,
    max_rooms: u32,
    start_ok: &dyn Fn(&MapGen) -> bool,
    count_fits: bool,
) -> (f64, Vec<f64>, Vec<MapGen<'a>>) {
    let mut mix = seed;
    let mut next = move || splitmix(&mut mix);
    // Each map starts from its own seed: the unit queues' order is random too.
    let mut alive: Vec<(MapGen, u32, bool)> = Vec::with_capacity(particles);
    for _ in 0..particles {
        let mut g = MapGen::new(Rng::with(generator, next()), caveinfo);
        if !g.place_start() {
            return (0.0, Vec::new(), Vec::new());
        }
        alive.push((g, 0, false));
    }
    let total = alive.len();
    alive.retain(|(g, _, _)| start_ok(g));
    let start_share = alive.len() as f64 / total as f64;
    if alive.is_empty() {
        return (0.0, vec![0.0], Vec::new());
    }
    alive = (0..particles).map(|i| alive[i % alive.len()].clone()).collect();
    let mut odds = start_share;
    let mut shares = vec![start_share];
    let last = levels.len() - 1;
    let mut closed_maps = Vec::new();
    for (level, (max_roomable, max_open)) in levels.into_iter().enumerate() {
        let mut hits = Vec::new();
        for (mut g, mut steps, mut closed) in alive {
            if steps > 0 {
                *g.rng_mut() = Rng::with(generator, next());
            }
            loop {
                let roomable = if count_fits { g.room_fits() } else { g.roomable_doors() };
                if closed || (max_open > 0 && g.open_doors().len() <= max_open && roomable <= max_roomable) {
                    hits.push((g, steps, closed));
                    break;
                }
                if steps >= 500 {
                    break;
                }
                steps += 1;
                closed = !g.placement_step(steps);
                if g.rooms_placed() >= caveinfo.num_rooms || g.rooms_placed() > max_rooms {
                    break;
                }
            }
        }
        let share = hits.len() as f64 / particles as f64;
        odds *= share;
        shares.push(share);
        if level == last {
            closed_maps = hits.into_iter().map(|h| h.0).collect();
            break;
        }
        if hits.is_empty() {
            break;
        }
        alive = (0..particles).map(|_| hits[(next() % hits.len() as u64) as usize].clone()).collect();
    }
    (odds, shares, closed_maps)
}
