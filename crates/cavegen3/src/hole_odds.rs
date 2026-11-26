//! Where the exit hole (or geyser) is likely to be, given what a runner has seen, when the seed
//! isn't known (Challenge Mode).
//!
//! The runner's view is a *reference* layout: some seed whose rooms around the pod look like
//! theirs. They mark which of its map units they've seen, and facts about spawn points in those
//! units (empty, holds something, the hole is or isn't there, nothing showed in the comedown).
//! [`tally`] then generates many other seeds and keeps the layouts that agree:
//!
//! - **Rooms**: the pod is in the same unit at the same spawn point, and every seen unit sits in
//!   the same place relative to the pod, the same way round. Positions are compared in the pod's
//!   frame, so a map turned as a whole still matches (the runner can't tell north).
//! - **Facts**: each marked spawn point (the same spot of the matching unit) agrees.
//!
//! For each agreeing layout the hole counts toward its spawn point when it's in a seen unit, or
//! else toward the exit of the seen area nearest to it (the open door of a seen unit with the
//! shortest walk through unseen units). Dividing by the number of agreeing layouts gives the
//! probabilities. The same is done for the geyser.
//!
//! Nothing here assumes how the hole is picked, so it covers everything the generator does: in
//! Challenge Mode the hole goes in an empty alcove or dead end weighted by the square root of
//! its unit's score plus 10 (`place_hole`), before any teki or treasure, and teki then keep 150
//! or 200 units away from it. A teki seen at a spot thus also says the hole isn't next to it.

use std::collections::{BTreeMap, BinaryHeap};

use crate::{
    Layout, SpawnObject,
    caveinfo::CaveInfo,
    generate::generate_partly,
    layout::PlacedUnit,
    rng::{Generator, Rng},
};

/// Teki the game draws during the comedown (plants and the fire, electric and gas hazards); every
/// other teki is hidden then. Lowercase internal names. Same list as `COMEDOWN_VISIBLE_TEKI` in
/// web/app.js and the CLI's render.rs.
pub const COMEDOWN_VISIBLE_TEKI: &[&str] = &[
    "chiyogami",
    "clover",
    "diodered",
    "diodegreen",
    "hikarikinoko",
    "kareooinu_l",
    "kareooinu_s",
    "margaret",
    "nekojarashi",
    "ooinu_l",
    "ooinu_s",
    "tanpopo",
    "tukushi",
    "wakame_l",
    "wakame_s",
    "watage",
    "zenmai",
    "hiba",
    "elechiba",
    "gashiba",
];

/// Whether the game draws an object while the pod comes down: everything but teki, and of teki
/// only the plants and hazards. A treasure a teki carries doesn't show either.
pub fn comedown_visible(so: &SpawnObject) -> bool {
    let teki = |name: &str| COMEDOWN_VISIBLE_TEKI.contains(&name.to_ascii_lowercase().as_str());
    match so {
        SpawnObject::Teki(info, _) => teki(&info.internal_name),
        SpawnObject::CapTeki(info, _) => teki(&info.internal_name),
        _ => true,
    }
}

/// Something the runner saw at a spawn point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fact {
    /// Nothing at all spawned there.
    Empty,
    /// Something other than the hole or geyser is there (a teki, treasure, plant...).
    Occupied,
    /// Nothing the comedown shows is there (it may hold a teki, which the comedown hides).
    NothingVisible,
    Hole,
    NoHole,
    Geyser,
    NoGeyser,
}

impl Fact {
    /// The one-letter code used in URLs and commands.
    pub fn code(self) -> char {
        match self {
            Fact::Empty => 'e',
            Fact::Occupied => 'o',
            Fact::NothingVisible => 'v',
            Fact::Hole => 'h',
            Fact::NoHole => 'n',
            Fact::Geyser => 'g',
            Fact::NoGeyser => 'x',
        }
    }

    pub fn from_code(c: char) -> Option<Fact> {
        Some(match c {
            'e' => Fact::Empty,
            'o' => Fact::Occupied,
            'v' => Fact::NothingVisible,
            'h' => Fact::Hole,
            'n' => Fact::NoHole,
            'g' => Fact::Geyser,
            'x' => Fact::NoGeyser,
            _ => return None,
        })
    }

    /// Whether a spawn point holding `contains` agrees with this fact.
    pub fn holds(self, contains: &[SpawnObject]) -> bool {
        let hole = contains.iter().any(|so| matches!(so, SpawnObject::Hole(_)));
        let geyser = contains.iter().any(|so| matches!(so, SpawnObject::Geyser(_)));
        match self {
            Fact::Empty => contains.is_empty(),
            Fact::Occupied => contains
                .iter()
                .any(|so| !matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_))),
            Fact::NothingVisible => !contains.iter().any(comedown_visible),
            Fact::Hole => hole,
            Fact::NoHole => !hole,
            Fact::Geyser => geyser,
            Fact::NoGeyser => !geyser,
        }
    }
}

/// What the runner has seen, in terms of the reference layout.
#[derive(Debug, Clone, Default)]
pub struct Observations {
    /// Indices of the reference's units the runner has seen. The pod's unit (0) always counts.
    pub seen: Vec<u32>,
    /// Facts about the reference's spawn points (by index). Facts about spawn points outside the
    /// seen units are ignored: only seen rooms carry over between layouts.
    pub facts: Vec<(u32, Fact)>,
}

impl Observations {
    /// Parses `seen` ("0,3,5", or "-" for none) and `facts` ("12e,40h", or "-").
    pub fn parse(seen: &str, facts: &str) -> Result<Observations, String> {
        let list = |s: &str| -> Vec<String> {
            let s = s.trim();
            if s.is_empty() || s == "-" {
                Vec::new()
            } else {
                s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
            }
        };
        let mut obs = Observations::default();
        for p in list(seen) {
            obs.seen.push(p.parse().map_err(|_| format!("bad unit number {p:?}"))?);
        }
        for p in list(facts) {
            let c = p.chars().last().unwrap();
            let fact = Fact::from_code(c).ok_or_else(|| format!("bad fact {p:?}"))?;
            let index = p[..p.len() - c.len_utf8()].parse().map_err(|_| format!("bad fact {p:?}"))?;
            obs.facts.push((index, fact));
        }
        Ok(obs)
    }

    /// The seen units with the pod's unit, sorted, without repeats.
    fn seen_units(&self) -> Vec<u32> {
        let mut seen = self.seen.clone();
        seen.push(0);
        seen.sort_unstable();
        seen.dedup();
        seen
    }
}

/// A unit's center in half grid cells, so it's a whole number whatever the unit's size.
fn center2(u: &PlacedUnit) -> (i32, i32) {
    (2 * u.x + u.unit.width as i32, 2 * u.z + u.unit.height as i32)
}

/// Turns a vector `k` quarter turns the way a unit's rotation does: (x, z) -> (-z, x).
fn turn((x, z): (i32, i32), k: u16) -> (i32, i32) {
    match k % 4 {
        0 => (x, z),
        1 => (-z, x),
        2 => (-x, -z),
        _ => (z, -x),
    }
}

/// The pod's spawn point as an index into its unit's spawn points.
fn ship_local(layout: &Layout) -> Option<u32> {
    let ship = layout.ship?;
    Some(ship - layout.units.first()?.first_spawnpoint)
}

/// For each of `units` (reference unit indices), the unit of `sample` in the same place relative
/// to the pod, or `None` if any is missing or the pod's spot differs.
pub fn match_units(reference: &Layout, units: &[u32], sample: &Layout) -> Option<Vec<u32>> {
    let (r0, s0) = (reference.units.first()?, sample.units.first()?);
    if r0.unit.name_id != s0.unit.name_id || ship_local(reference)? != ship_local(sample)? {
        return None;
    }
    // Quarter turns from the sample's frame to the reference's.
    let k = (r0.unit.rotation + 4 - s0.unit.rotation) % 4;
    let (rc0, sc0) = (center2(r0), center2(s0));
    let mut out = Vec::with_capacity(units.len());
    for &ri in units {
        let r = reference.units.get(ri as usize)?;
        let want = center2(r);
        let found = sample.units.iter().position(|s| {
            if s.unit.name_id != r.unit.name_id || (s.unit.rotation + k) % 4 != r.unit.rotation {
                return false;
            }
            let c = center2(s);
            let (dx, dz) = turn((c.0 - sc0.0, c.1 - sc0.1), k);
            (rc0.0 + dx, rc0.1 + dz) == want
        })?;
        out.push(found as u32);
    }
    Some(out)
}

/// The sample's spawn point matching the reference's spawn point `ri`, given the unit matches
/// from [`match_units`] (`None` if `ri` isn't in a matched unit).
fn map_spawnpoint(reference: &Layout, units: &[u32], map: &[u32], sample: &Layout, ri: u32) -> Option<usize> {
    let sp = reference.spawnpoints.get(ri as usize)?;
    let pos = units.iter().position(|&u| u == sp.unit)?;
    let local = ri - reference.units[sp.unit as usize].first_spawnpoint;
    Some((sample.units[map[pos] as usize].first_spawnpoint + local) as usize)
}

/// Re-expresses observations made on `reference` in terms of `other`, when its rooms match
/// (`None` otherwise): for moving to another layout without losing the marks.
pub fn remap(reference: &Layout, obs: &Observations, other: &Layout) -> Option<Observations> {
    let units = obs.seen_units();
    let map = match_units(reference, &units, other)?;
    let facts = obs
        .facts
        .iter()
        .filter_map(|&(ri, fact)| Some((map_spawnpoint(reference, &units, &map, other, ri)? as u32, fact)))
        .collect();
    Some(Observations { seen: map, facts })
}

/// Counts toward one exit object's location.
#[derive(Debug, Clone, Default)]
pub struct Where {
    /// Reference spawn point index -> layouts with it there.
    pub spots: BTreeMap<u32, f64>,
    /// Reference door index (an open door of a seen unit) -> layouts with it beyond that door.
    pub exits: BTreeMap<u32, f64>,
    /// Layouts where it's beyond the seen units but no exit reaches it (shouldn't happen).
    pub other: f64,
    /// Layouts without one.
    pub none: f64,
}

/// The result of [`tally`]; counts add up across calls with [`Tally::add`].
#[derive(Debug, Clone, Default)]
pub struct Tally {
    pub tested: u64,
    /// Layouts whose seen rooms match.
    pub matched: u64,
    /// Matching layouts that also agree with every fact.
    pub consistent: u64,
    pub hole: Where,
    pub geyser: Where,
    /// The first few consistent seeds, for looking at.
    pub examples: Vec<u64>,
}

impl Tally {
    pub fn add(&mut self, other: &Tally, max_examples: usize) {
        self.tested += other.tested;
        self.matched += other.matched;
        self.consistent += other.consistent;
        for (a, b) in [(&mut self.hole, &other.hole), (&mut self.geyser, &other.geyser)] {
            for (k, v) in &b.spots {
                *a.spots.entry(*k).or_default() += v;
            }
            for (k, v) in &b.exits {
                *a.exits.entry(*k).or_default() += v;
            }
            a.other += b.other;
            a.none += b.none;
        }
        for &s in &other.examples {
            if self.examples.len() < max_examples {
                self.examples.push(s);
            }
        }
    }
}

/// The open doors of the seen units in the reference: (reference door, unit position in `units`,
/// door index within its unit).
fn seen_exits(reference: &Layout, units: &[u32]) -> Vec<(u32, usize, usize)> {
    let mut out = Vec::new();
    for (pos, &ui) in units.iter().enumerate() {
        let u = &reference.units[ui as usize];
        for (local, d) in u.door_indices().enumerate() {
            let beyond = reference.doors[d].adjacent.map(|a| reference.doors[a as usize].unit);
            if beyond.is_some_and(|b| !units.contains(&b)) {
                out.push((d as u32, pos, local));
            }
        }
    }
    out
}

/// Which exits of the seen area the object at sample spawn point `target` is nearest to, by
/// walking distance (door links within units, then straight to the spot) through unseen units.
/// Ties share the count.
fn nearest_exits(sample: &Layout, seen_sample: &[u32], exits: &[(u32, usize)], target: usize) -> Vec<u32> {
    let n = sample.doors.len();
    let mut dist = vec![f32::INFINITY; n];
    let mut label: Vec<Vec<u32>> = vec![Vec::new(); n];
    // Max-heap on negated, scaled distances.
    let mut heap = BinaryHeap::new();
    for &(ref_door, sample_door) in exits {
        let Some(adj) = sample.doors[sample_door].adjacent else { continue };
        let adj = adj as usize;
        if dist[adj] > 0.0 {
            dist[adj] = 0.0;
            label[adj].clear();
        }
        if !label[adj].contains(&ref_door) {
            label[adj].push(ref_door);
        }
        heap.push((0i64, adj));
    }
    let key = |d: f32| -((d * 16.0) as i64);
    while let Some((negd, d)) = heap.pop() {
        if negd != key(dist[d]) {
            continue;
        }
        let door = &sample.doors[d];
        let unit = &sample.units[door.unit as usize];
        let relax =
            |to: usize, nd: f32, from: &Vec<u32>, heap: &mut BinaryHeap<(i64, usize)>, dist: &mut Vec<f32>, label: &mut Vec<Vec<u32>>| {
                if nd < dist[to] - 1e-3 {
                    dist[to] = nd;
                    label[to] = from.clone();
                    heap.push((key(nd), to));
                } else if (nd - dist[to]).abs() <= 1e-3 {
                    for l in from {
                        if !label[to].contains(l) {
                            label[to].push(*l);
                        }
                    }
                }
            };
        let from = label[d].clone();
        for link in &door.door.door_links {
            let to = unit.first_door as usize + link.door_id;
            relax(to, dist[d] + link.distance, &from, &mut heap, &mut dist, &mut label);
        }
        if let Some(adj) = door.adjacent {
            let adj = adj as usize;
            if !seen_sample.contains(&sample.doors[adj].unit) {
                relax(adj, dist[d], &from, &mut heap, &mut dist, &mut label);
            }
        }
    }
    let sp = &sample.spawnpoints[target];
    let unit = &sample.units[sp.unit as usize];
    let mut best = f32::INFINITY;
    let mut out: Vec<u32> = Vec::new();
    for d in unit.door_indices() {
        if !dist[d].is_finite() {
            continue;
        }
        let total = dist[d] + sample.doors[d].center().p2_dist(&sp.pos);
        if total < best - 1e-3 {
            best = total;
            out = label[d].clone();
        } else if (total - best).abs() <= 1e-3 {
            for l in &label[d] {
                if !out.contains(l) {
                    out.push(*l);
                }
            }
        }
    }
    out
}

/// Generates `seeds` and counts where the hole and geyser are in the layouts that agree with
/// `obs` on `reference` (see the module docs). Keeps up to `max_examples` agreeing seeds.
pub fn tally(
    reference: &Layout,
    obs: &Observations,
    caveinfo: &CaveInfo,
    generator: Generator,
    seeds: impl IntoIterator<Item = u64>,
    max_examples: usize,
) -> Tally {
    let units = obs.seen_units();
    let exits = seen_exits(reference, &units);
    // Facts on seen units only, as (unit position in `units`, spawn point index within it).
    let facts: Vec<(usize, u32, Fact)> = obs
        .facts
        .iter()
        .filter_map(|&(ri, fact)| {
            let sp = reference.spawnpoints.get(ri as usize)?;
            let pos = units.iter().position(|&u| u == sp.unit)?;
            Some((pos, ri - reference.units[sp.unit as usize].first_spawnpoint, fact))
        })
        .collect();

    // Layouts with another unit for the pod's are dropped as soon as it's placed, and teki and
    // treasures are only placed when a fact needs them.
    let first_unit = reference.units.first().map(|u| u.unit.name_id);
    let exits_only = facts
        .iter()
        .all(|f| matches!(f.2, Fact::Hole | Fact::NoHole | Fact::Geyser | Fact::NoGeyser));

    let mut t = Tally::default();
    for seed in seeds {
        t.tested += 1;
        let Some(sample) = generate_partly(Rng::with(generator, seed), caveinfo, first_unit, exits_only) else {
            continue;
        };
        let Some(map) = match_units(reference, &units, &sample) else {
            continue;
        };
        t.matched += 1;
        let agrees = facts.iter().all(|&(pos, local, fact)| {
            let si = sample.units[map[pos] as usize].first_spawnpoint + local;
            fact.holds(&sample.spawnpoints[si as usize].contains)
        });
        if !agrees {
            continue;
        }
        t.consistent += 1;
        if t.examples.len() < max_examples {
            t.examples.push(seed);
        }
        for (object, w) in [(sample.hole, &mut t.hole), (sample.geyser, &mut t.geyser)] {
            let Some(si) = object else {
                w.none += 1.0;
                continue;
            };
            let sp = &sample.spawnpoints[si as usize];
            if let Some(pos) = map.iter().position(|&u| u == sp.unit) {
                let local = si - sample.units[sp.unit as usize].first_spawnpoint;
                let ri = reference.units[units[pos] as usize].first_spawnpoint + local;
                *w.spots.entry(ri).or_default() += 1.0;
                continue;
            }
            let sample_exits: Vec<(u32, usize)> = exits
                .iter()
                .map(|&(rd, pos, local)| (rd, sample.units[map[pos] as usize].first_door as usize + local))
                .collect();
            let nearest = nearest_exits(&sample, &map, &sample_exits, si as usize);
            if nearest.is_empty() {
                w.other += 1.0;
            } else {
                let share = 1.0 / nearest.len() as f64;
                for rd in nearest {
                    *w.exits.entry(rd).or_default() += share;
                }
            }
        }
    }
    t
}
