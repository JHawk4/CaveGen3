//! How likely each spot was to get the exit (the hole, or the geyser on a final floor), seen from
//! the finished layout: for Challenge Mode, where the seed isn't known but the map and what's on
//! it are.
//!
//! The game places the exit right after the pod, before any teki or treasure (`place_hole`): in
//! an empty alcove (group 4) or dead end (group 9), picked at random with weight sqrt(unit score)
//! + 10 in Challenge Mode (its `hole_score`). Everything placed afterwards had one spot fewer to
//! choose from, so what's on the map says something about where the exit is. By Bayes' rule each
//! candidate spot `h` gets its weight times the chance of what's seen if the exit were at `h`:
//!
//! - **Spots holding something else** can't have it.
//! - **Hard and special teki** pick uniformly among their spots at least 200 (hard) or 150
//!   (special) from the exit: a teki that close rules `h` out, and the fewer spots `h` leaves
//!   them, the likelier the teki seen.
//! - **Treasures** are drawn one at a time, weighted, from the empty treasure spots: room spots
//!   and item alcoves. An exit in an item alcove takes it out of the draw, so the treasures seen
//!   came from a smaller total. Worked out exactly for the whole set of treasures.
//! - **Cap teki** go into item alcoves: grounded ones into empty alcoves (minimum amounts, then
//!   filler in every alcove left), falling ones into any alcove without a Candypop Bud or the exit.
//!   Which alcoves they took is treated as random (the game goes in unit order, which a runner
//!   can't see), so with a fixed number an exit alcove leaves them fewer to choose from, and with
//!   filler every alcove they left must hold the exit.
//! - **Gates** go first in front of item alcoves holding a treasure, the exit or a grounded cap
//!   teki, then on the lowest-scored door of each room, then (80%) on a room door weighted towards
//!   low door scores, else on any door. The chance of the whole set of gates seen is worked out
//!   for each `h`: a gate in front of an alcove that looks empty usually means the exit is there.
//!
//! A kind of evidence that no candidate fits (the generator did something this doesn't model) is
//! left out. In story mode the highest score always wins, so there are no odds to give.

use std::collections::BTreeMap;

use crate::{
    Layout, SpawnObject,
    caveinfo::{CapInfo, RoomType},
};

fn is_exit(so: &SpawnObject) -> bool {
    matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_))
}

/// The chance of the exit being at each spawn point of `layout` (only spots with a chance are
/// listed), given everything else on the map; empty outside Challenge Mode. See the module docs.
pub fn exit_odds(layout: &Layout) -> BTreeMap<u32, f64> {
    if !layout.caveinfo.is_challenge_mode() {
        return BTreeMap::new();
    }
    let spots: Vec<usize> = layout
        .spawnpoints
        .iter()
        .enumerate()
        .filter(|(_, sp)| matches!(sp.group(), 4 | 9) && sp.hole_score > 0 && sp.contains.iter().all(is_exit))
        .map(|(i, _)| i)
        .collect();
    let mut weights: Vec<f64> = spots.iter().map(|&i| layout.spawnpoints[i].hole_score as f64).collect();
    let evidence = Evidence::new(layout);
    for factor in [
        evidence.teki(&spots),
        evidence.treasures(&spots),
        evidence.cap_teki(&spots, false),
        evidence.cap_teki(&spots, true),
        evidence.gates(&spots),
    ] {
        let next: Vec<f64> = weights.iter().zip(&factor).map(|(w, f)| w * f).collect();
        let total: f64 = next.iter().sum();
        // Evidence no candidate fits is left out; the rest is rescaled to keep the numbers in range.
        if total > 0.0 && total.is_finite() {
            weights = next.iter().map(|w| w / total).collect();
        }
    }
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return BTreeMap::new();
    }
    spots
        .iter()
        .zip(&weights)
        .filter(|(_, w)| **w > 0.0)
        .map(|(&i, &w)| (i as u32, w / total))
        .collect()
}

/// The rest of the layout, for working out how likely it is with the exit at each candidate.
struct Evidence<'l, 'a> {
    layout: &'l Layout<'a>,
    /// Item alcoves: their alcove spawn point and their door.
    alcoves: Vec<(usize, usize)>,
}

impl<'l, 'a> Evidence<'l, 'a> {
    fn new(layout: &'l Layout<'a>) -> Self {
        let alcoves = layout
            .units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::DeadEnd && u.unit.unit_folder_name.contains("item"))
            .filter_map(|u| {
                let si = u.spawnpoint_indices().find(|&i| layout.spawnpoints[i].group() == 9)?;
                Some((si, u.first_door as usize))
            })
            .collect();
        Evidence { layout, alcoves }
    }

    fn holds(&self, si: usize, f: impl Fn(&SpawnObject) -> bool) -> bool {
        self.layout.spawnpoints[si].contains.iter().any(f)
    }

    /// Hard and special teki: uniform picks among their spots far enough from the exit.
    fn teki(&self, spots: &[usize]) -> Vec<f64> {
        let l = self.layout;
        let ship = l.ship.map(|s| l.spawnpoints[s as usize].pos);
        let mut out = vec![1.0; spots.len()];
        for (group, gap) in [(1u16, 200.0), (8, 150.0)] {
            let pool: Vec<usize> = l
                .units
                .iter()
                .filter(|u| u.unit.room_type == RoomType::Room)
                .flat_map(|u| u.spawnpoint_indices())
                .filter(|&i| l.spawnpoints[i].group() == group && ship.is_none_or(|s| s.p2_dist(&l.spawnpoints[i].pos) >= 300.0))
                .collect();
            let placed: Vec<usize> = pool.iter().copied().filter(|&i| self.holds(i, |so| matches!(so, SpawnObject::Teki(..)))).collect();
            for (k, &h) in spots.iter().enumerate() {
                let hp = l.spawnpoints[h].pos;
                let far = |i: &usize| l.spawnpoints[*i].pos.p2_dist(&hp) >= gap;
                out[k] *= if placed.iter().all(far) {
                    1.0 / choose(pool.iter().filter(|i| far(i)).count(), placed.len())
                } else {
                    0.0
                };
            }
        }
        out
    }

    /// Treasures: weighted draws without replacement from the empty treasure spots.
    fn treasures(&self, spots: &[usize]) -> Vec<f64> {
        let l = self.layout;
        // Every treasure spot and its weight in Challenge Mode.
        let mut slots: Vec<(usize, f64)> = Vec::new();
        for u in &l.units {
            if u.unit.room_type == RoomType::Room {
                let room: Vec<usize> = u.spawnpoint_indices().filter(|&i| l.spawnpoints[i].group() == 2).collect();
                slots.extend(room.iter().map(|&i| (i, (1 + u.total_score / room.len() as i32) as f64)));
            }
        }
        for &(si, _) in &self.alcoves {
            let u = &l.units[l.spawnpoints[si].unit as usize];
            slots.push((si, (1 + u.total_score * 10) as f64));
        }
        let item = |so: &SpawnObject| matches!(so, SpawnObject::Item(_));
        let drawn: Vec<f64> = slots.iter().filter(|s| self.holds(s.0, item)).map(|s| s.1).collect();
        // Spots that were empty but got no treasure, counting the exit's as empty (cap teki come
        // later): the exit is at most one of them.
        let left: Vec<(usize, f64)> = slots
            .iter()
            .copied()
            .filter(|s| l.spawnpoints[s.0].contains.iter().all(|so| is_exit(so) || matches!(so, SpawnObject::CapTeki(..))))
            .collect();
        let left_total: f64 = left.iter().map(|s| s.1).sum();
        let mut cache: BTreeMap<u64, f64> = BTreeMap::new();
        spots
            .iter()
            .map(|&h| {
                let rest = left_total - left.iter().find(|s| s.0 == h).map_or(0.0, |s| s.1);
                *cache.entry(rest.to_bits()).or_insert_with(|| drawn_first(&drawn, rest))
            })
            .collect()
    }

    /// Cap teki of one kind (grounded or falling), which alcoves they took treated as random.
    fn cap_teki(&self, spots: &[usize], falling: bool) -> Vec<f64> {
        let l = self.layout;
        let class = |t: &CapInfo| if falling { t.is_falling() && !t.is_candypop() } else { !t.is_falling() || t.is_candypop() };
        // The alcoves they could take with no exit anywhere: grounded ones the empty ones (falling
        // cap teki come later), falling ones any without a Candypop Bud.
        let pool: Vec<usize> = self
            .alcoves
            .iter()
            .map(|a| a.0)
            .filter(|&si| {
                if falling {
                    !self.holds(si, |so| matches!(so, SpawnObject::CapTeki(t, _) if t.is_candypop()))
                } else {
                    l.spawnpoints[si].contains.iter().all(|so| is_exit(so) || matches!(so, SpawnObject::CapTeki(..)))
                }
            })
            .collect();
        let taken = pool.iter().filter(|&&si| self.holds(si, |so| matches!(so, SpawnObject::CapTeki(t, _) if class(t)))).count();
        let limit = cap_alcove_limit(&l.caveinfo.cap_info, class);
        spots
            .iter()
            .map(|&h| {
                let n = pool.len() - usize::from(pool.contains(&h));
                match limit {
                    // They stop only when they run out, so fewer means every alcove was taken.
                    Some(m) if taken >= m => 1.0 / choose(n, taken),
                    _ => f64::from(u8::from(taken == n)),
                }
            })
            .collect()
    }

    /// Gates: the chance of the gates seen, placed the way `gate_spot` places them.
    fn gates(&self, spots: &[usize]) -> Vec<f64> {
        let l = self.layout;
        let pair = |d: usize| l.doors[d].adjacent.map_or(d, |a| d.min(a as usize));
        let mut gates: Vec<usize> = (0..l.doors.len())
            .filter(|&d| matches!(l.doors[d].seam, Some(SpawnObject::Gate(..))))
            .map(pair)
            .collect();
        gates.sort_unstable();
        gates.dedup();
        if gates.is_empty() || gates.len() > 14 {
            return vec![1.0; spots.len()];
        }
        // Alcoves the first step aims for, apart from the exit's.
        let filled: Vec<bool> = self
            .alcoves
            .iter()
            .map(|&(si, _)| {
                self.holds(si, |so| match so {
                    SpawnObject::CapTeki(t, _) => !t.is_falling(),
                    SpawnObject::Item(_) => true,
                    _ => false,
                }) && !self.holds(si, |so| matches!(so, SpawnObject::CapTeki(t, _) if t.is_candypop() && t.is_falling()))
            })
            .collect();
        let mut cache: BTreeMap<Option<usize>, f64> = BTreeMap::new();
        spots
            .iter()
            .map(|&h| {
                let at = self.alcoves.iter().position(|a| a.0 == h);
                *cache.entry(at).or_insert_with(|| {
                    let mut filled = filled.clone();
                    if let Some(a) = at {
                        filled[a] = true;
                    }
                    self.gate_set_chance(&gates, &filled, &pair)
                })
            })
            .collect()
    }

    /// The chance the gates go on exactly `gates` (door pairs, by their lower door), in any order.
    fn gate_set_chance(&self, gates: &[usize], filled: &[bool], pair: &dyn Fn(usize) -> usize) -> f64 {
        let l = self.layout;
        let k = gates.len();
        let seam_teki: Vec<bool> = l.doors.iter().map(|d| matches!(d.seam, Some(SpawnObject::Teki(..)))).collect();
        let all = (1usize << k) - 1;
        let mut chance = vec![0.0; all + 1];
        chance[0] = 1.0;
        for set in 0..all {
            if chance[set] == 0.0 {
                continue;
            }
            let free = |d: usize| !seam_teki[d] && !(0..k).any(|g| set & (1 << g) != 0 && gates[g] == pair(d));
            let next = self.gate_step(filled, &free);
            for (g, &p) in gates.iter().enumerate() {
                if set & (1 << g) == 0 {
                    let q: f64 = next.iter().filter(|(d, _)| pair(*d) == p).map(|(_, q)| q).sum();
                    chance[set | (1 << g)] += chance[set] * q;
                }
            }
        }
        chance[all]
    }

    /// Where the next gate goes, as chances over doors, given which doors are free.
    fn gate_step(&self, filled: &[bool], free: &dyn Fn(usize) -> bool) -> Vec<(usize, f64)> {
        let l = self.layout;
        // 1. In front of a filled item alcove.
        let first: Vec<usize> = self.alcoves.iter().zip(filled).filter(|(a, f)| **f && free(a.1)).map(|(a, _)| a.1).collect();
        if !first.is_empty() {
            let q = 1.0 / first.len() as f64;
            return first.into_iter().map(|d| (d, q)).collect();
        }
        // 2. The lowest-scored door of each room but the pod's.
        let ship_unit = l.ship.map(|s| l.spawnpoints[s as usize].unit as usize);
        for (ui, u) in l.units.iter().enumerate() {
            if u.unit.room_type != RoomType::Room || Some(ui) == ship_unit {
                continue;
            }
            let mut min = (128_000, None);
            for d in u.door_indices() {
                if l.doors[d].door_score < min.0 {
                    min = (l.doors[d].door_score, Some(d));
                }
            }
            if let (s, Some(d)) = min
                && s < 128_000
                && free(d)
            {
                return vec![(d, 1.0)];
            }
        }
        let weighted = |spots: Vec<(usize, f64)>, scale: f64| -> Vec<(usize, f64)> {
            let total: f64 = spots.iter().map(|s| s.1).sum();
            if total <= 0.0 { Vec::new() } else { spots.into_iter().map(|(d, w)| (d, scale * w / total)).collect() }
        };
        // 3. (80% of the time) A room door, weighted towards low door scores.
        let room_doors: Vec<usize> = l
            .units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::Room)
            .flat_map(|u| u.door_indices())
            .filter(|&d| free(d))
            .collect();
        let max = room_doors.iter().map(|&d| l.doors[d].door_score).fold(0, i32::max);
        let third = weighted(room_doors.iter().map(|&d| (d, (max + 1 - l.doors[d].door_score) as f64)).collect(), 0.8);
        // 4. Any door.
        let any: Vec<(usize, f64)> = l
            .units
            .iter()
            .flat_map(|u| {
                let n = u.unit.doors.len();
                let w = if u.unit.room_type == RoomType::Hallway { (10 / n) as f64 } else { n as f64 };
                u.door_indices().map(move |d| (d, w))
            })
            .filter(|&(d, _)| free(d))
            .collect();
        let fourth = weighted(any, if third.is_empty() { 1.0 } else { 0.2 });
        third.into_iter().chain(fourth).collect()
    }
}

/// How many alcoves cap teki of one kind fill with their minimum amounts
/// (`choose_rand_cap_teki`), or `None` if they have filler and fill every alcove they can.
fn cap_alcove_limit(caps: &[CapInfo], class: impl Fn(&CapInfo) -> bool) -> Option<usize> {
    if caps.iter().any(|t| class(t) && t.filler_distribution_weight > 0) {
        return None;
    }
    let (mut spawned, mut alcoves) = (0u32, 0usize);
    loop {
        let mut cumulative = 0;
        let mut placed = None;
        for t in caps.iter().filter(|t| class(t)) {
            cumulative += t.minimum_amount;
            if spawned < cumulative {
                placed = Some(if t.group == 0 && spawned + 1 < cumulative { 2 } else { 1 });
                break;
            }
        }
        let Some(n) = placed else { return Some(alcoves) };
        spawned += n;
        alcoves += 1;
    }
}

/// n choose k, as a float (infinite when k > n, so that 1 / it is 0).
fn choose(n: usize, k: usize) -> f64 {
    if k > n {
        return f64::INFINITY;
    }
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// The chance that weighted draws without replacement take exactly the spots weighing `drawn`
/// first, in any order, when the spots not drawn weigh `rest` in all. With exponential clocks
/// (each spot rings at the rate of its weight; the draw order is the ringing order) that's
/// ∫ rest·e^(−rest·t) · Π(1 − e^(−w·t)) dt, done numerically.
fn drawn_first(drawn: &[f64], rest: f64) -> f64 {
    if drawn.is_empty() || rest <= 0.0 {
        return 1.0;
    }
    // In s = rest·t the integrand is e^(−s) Π(1 − e^(−w·s/rest)), which is gone by s = 4k + 60.
    let end = drawn.len() as f64 * 4.0 + 60.0;
    let n = 20_000;
    let h = end / n as f64;
    let f = |s: f64| (-s).exp() * drawn.iter().map(|w| -(-w * s / rest).exp_m1()).product::<f64>();
    let mut sum = f(0.0) + f(end);
    for i in 1..n {
        sum += f(i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 };
    }
    sum * h / 3.0
}
