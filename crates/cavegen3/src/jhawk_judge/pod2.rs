//! judge2: the Pay off Debt judge refit against timed runs. Not part of JHawk's CaveGen.
//!
//! The `pod` judge assumes the runner collects every counted treasure. judge2 rebuilds the
//! same parts (walking, carry, hazards, gates, Glutton's Kitchen adjustments, missing pokos)
//! for each choice of which counted treasures the runner takes, weighs them with weights fitted
//! to real Pay off Debt runs, and averages the choices' scores with weights exp(-score / 5 s),
//! so a treasure that isn't worth its trip counts as probably skipped. It drops `pod`'s credit
//! for bug pokos and optional treasures (runners take neither in practice) and adds 100 s when
//! SH-6's lens is taken while its carry route crosses water (runners skip it then).
//!
//! The parts follow the `pod` judge line by line, including the graph queries whose results
//! only feed diagnostics: CaveGen's pointToSegmentDist keeps state between calls (see
//! `graph.rs`), so the sequence of queries is kept as it was when the weights were fitted.

use std::collections::HashSet;

use super::graph::{Graph, V3, dist};
use super::{Ctx, JhawkJudge, Placed, carry_info, difficulty, names, pokos};
use crate::layout::Layout;

/// judge2's score of one layout.
#[derive(Debug, Clone)]
pub struct Judge2 {
    /// Expected net seconds (lower is better): the choices' scores, weighted by exp(-score / 5).
    pub score: f64,
    /// The take/skip choices considered, taking everything first.
    pub choices: Vec<Judge2Choice>,
}

/// One choice of which counted treasures the runner leaves behind.
#[derive(Debug, Clone)]
pub struct Judge2Choice {
    /// Internal names of the treasures skipped (none: take them all).
    pub skipped: Vec<String>,
    /// This choice's score in seconds.
    pub score: f64,
    /// How likely the runner makes this choice (the choices' weights, normalized).
    pub p: f64,
}

impl Judge2 {
    /// Each counted treasure (internal name, in caveinfo order) with the chance it is skipped.
    pub fn skip_chances(&self) -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = Vec::new();
        for c in self.choices.iter() {
            for s in c.skipped.iter() {
                match out.iter_mut().find(|(n, _)| n == s) {
                    Some(e) => e.1 += c.p,
                    None => out.push((s.clone(), c.p)),
                }
            }
        }
        out
    }
}

fn ctx<'a>(j: &'a JhawkJudge, layout: &Layout) -> Ctx<'a> {
    let placed = Placed::from_layout(layout);
    let start_pos = placed.sps[placed.start].pos;
    let graph = Graph::build(layout, start_pos);
    Ctx { j, p: placed, g: graph }
}

/// The parts of one take/skip choice, in seconds.
#[derive(Debug, Clone, Default)]
struct Parts {
    walking: f64,
    carry: f64,
    hazard: f64,
    gate: f64,
    /// `pod`'s optional-treasure credit (weight 0 in judge2)
    optional: f64,
    /// (expected pokos - pokos of the taken treasures) / 2
    poko: f64,
    /// bug pokos / 2 (`pod` subtracts this; weight 0 in judge2)
    bug: f64,
    adj: f64,
    taken: i32,
    /// SH-6: water the lens (bey_goma) carry route must cross, if it is taken
    lens_cross: f64,
    skipped: Vec<String>,
}

/// The counted (neither ignored nor optional) treasures: (name, spawn point, dug up?).
fn counted(c: &Ctx) -> Vec<(String, usize, bool)> {
    let n = names();
    c.p.items
        .iter()
        .map(|i| (i.name.to_ascii_lowercase(), i.sp, true))
        .chain(
            c.p.tekis
                .iter()
                .filter_map(|t| t.item_inside.as_ref().map(|nm| (nm.to_ascii_lowercase(), t.sp, false))),
        )
        .filter(|(name, ..)| !n.ignore_treasures_pod.contains(name.as_str()) && !n.optional_treasures_pod.contains(name.as_str()))
        .collect()
}

/// `pod`'s parts when the counted treasures with `skip[i]` set are left behind.
fn parts(j: &JhawkJudge, layout: &Layout, skip: &[bool]) -> Parts {
    let mut c = ctx(j, layout);
    let n = names();
    let rects = water_rects(layout);
    let all = counted(&c);
    let skipped = |i: usize| skip.get(i).copied().unwrap_or(false);
    let take: Vec<&(String, usize, bool)> = all.iter().enumerate().filter(|(i, _)| !skipped(*i)).map(|(_, t)| t).collect();
    let mut out = Parts {
        skipped: all
            .iter()
            .enumerate()
            .filter(|(i, _)| skipped(*i))
            .map(|(_, t)| t.0.clone())
            .collect(),
        ..Default::default()
    };

    let num_treasures_collected = take.len() as i32;
    let num_pokos_collected: i32 = take.iter().map(|t| pokos(&t.0)).sum();
    let targets: Vec<usize> = take.iter().map(|t| t.1).collect();

    // carry
    let mut pikmin_seconds = 0.0f32;
    let mut active_pikmin = 0.0f32;
    for (name, sp, dig) in take.iter() {
        c.mark(*sp);
        active_pikmin += carry_info(name).max_carry as f32;
        pikmin_seconds += j.story_carry(&mut c, name, *sp, *dig, j.piki_carry_value);
        let p = c.sp_pos(*sp);
        let (cross, _) = route_cross(&mut c, &rects, p);
        if name == "bey_goma" {
            out.lens_cross = cross as f64;
        }
    }
    let carry_seconds = if pikmin_seconds == 0.0 {
        0.0
    } else {
        pikmin_seconds / (j.piki_count as f32).min(active_pikmin)
    };

    let mut walking_seconds = c.walking_mst(&targets);
    let hole = c.p.hole.or(c.p.geyser);
    if let Some(h) = hole {
        let d = c.dist_to_start(h);
        walking_seconds += d / j.olimar_speed_cm / (1 + num_treasures_collected) as f32;
        let p = c.sp_pos(h);
        route_cross(&mut c, &rects, p); // diagnostic only (see the module notes)
    } else {
        walking_seconds += 300.0;
    }

    // teki in the way, bug pokos
    let mut hazard_seconds = 0.0f32;
    let mut bug_pokos = 0;
    c.compute_carryable("");
    for t in 0..c.p.tekis.len() {
        if c.teki_in_the_way(t) {
            let name = c.p.tekis[t].name.to_ascii_lowercase();
            hazard_seconds += difficulty(&name).sec as f32 / 2.0;
            let sp = c.p.tekis[t].sp;
            if !n.no_carcass.contains(name.as_str()) && c.dist_to_start(sp) / j.olimar_speed_cm < walking_seconds * 0.4 {
                bug_pokos += pokos(&name);
            }
        }
    }

    // optional treasures, as `pod`
    let mut optional_seconds = 0.0f32;
    let dig_divisor = (j.piki_count / (num_treasures_collected + 1)) as f32;
    for k in 0..c.p.items.len() {
        let (name, sp) = (c.p.items[k].name.to_ascii_lowercase(), c.p.items[k].sp);
        if !n.optional_treasures_pod.contains(name.as_str()) {
            continue;
        }
        let info = carry_info(&name);
        let dig = c.dig_time(&name);
        let min_dist_visited = JhawkJudge::min_dist_to_visited(&c, sp, true);
        let mut gate_check = 0.0f32;
        if !c.p.gates.is_empty() && c.p.sps[sp].is_alcove {
            if let Some(door) = c.p.first_door_pos[c.p.sps[sp].unit.unwrap()] {
                for g in c.p.gates.iter() {
                    let gp = c.sp_pos(g.sp);
                    if gp.x == door.x && gp.z == door.z {
                        gate_check += c.gate_cost(g.life, j.piki_count as f32);
                    }
                }
            }
        }
        let carry_value = if j.cave_short_name == "FC" && j.floor == 6 {
            2.4
        } else {
            j.piki_carry_value
        };
        let d = c.dist_to_start(sp);
        let time_needed = 10.0
            + 2.0 * min_dist_visited / j.olimar_speed_cm
            + 3.0
            + dig / dig_divisor
            + gate_check
            + d * 8.5 / Ctx::carry_speed(carry_value, info.max_carry, info.min_carry, info.max_carry);
        optional_seconds -= JhawkJudge::optional_time_save(pokos(&name), time_needed);
    }
    for k in 0..c.p.tekis.len() {
        let Some(name) = c.p.tekis[k].item_inside.as_ref().map(|nm| nm.to_ascii_lowercase()) else {
            continue;
        };
        if !n.optional_treasures_pod.contains(name.as_str()) {
            continue;
        }
        let (teki_name, sp) = (c.p.tekis[k].name.clone(), c.p.tekis[k].sp);
        let info = carry_info(&name);
        let min_dist_visited = JhawkJudge::min_dist_to_visited(&c, sp, false);
        let d = c.dist_to_start(sp);
        let time_needed = 10.0
            + difficulty(&teki_name).sec as f32
            + 2.0 * min_dist_visited / j.olimar_speed_cm
            + 3.0
            + 0.0 / dig_divisor
            + d * 8.5 / Ctx::carry_speed(j.piki_carry_value, info.max_carry, info.min_carry, info.max_carry);
        optional_seconds -= JhawkJudge::optional_time_save(pokos(&name), time_needed);
    }

    // teki in the way of the hole route only: diagnostic only (see the module notes)
    if hole.is_some() {
        let before: Vec<bool> = (0..c.p.tekis.len()).map(|t| c.teki_in_the_way(t)).collect();
        c.compute_carryable("h");
        for t in 0..c.p.tekis.len() {
            if !before[t] {
                c.teki_in_the_way(t);
            }
        }
    }

    // gates on the routes of the taken treasures and the hole
    let mut gate_seconds = 0.0f32;
    if !c.p.gates.is_empty() {
        c.compute_carryable("h");
        for g in 0..c.p.gates.len() {
            if c.gate_in_the_way(g) {
                gate_seconds += c.gate_cost(c.p.gates[g].life, j.piki_count as f32);
            }
        }
        // the hole's own gates: diagnostic only (see the module notes)
        if let Some(h) = hole {
            let p = c.sp_pos(h);
            let (_, wps) = route_cross(&mut c, &rects, p);
            gates_on(&mut c, &wps);
        }
    }

    // Glutton's Kitchen adjustments, as `pod` (high treasures and breadbugs only for taken ones)
    let mut adj_seconds = 0.0f32;
    if j.cave_short_name == "GK" {
        if j.floor == 2 {
            for i in c.p.items.iter() {
                if i.name.eq_ignore_ascii_case("g_futa_kyusyu") && !out.skipped.iter().any(|s| s == "g_futa_kyusyu") {
                    let idx = c.p.sps[i.sp].list_idx;
                    if idx == 6 || idx == 7 {
                        walking_seconds = 4.0;
                    }
                }
            }
            hazard_seconds = 0.0;
        }
        let taken_sps: Vec<usize> = take.iter().map(|t| t.1).collect();
        let considered = |name: &str, sp: usize| {
            !n.ignore_treasures_pod.contains(name) && (taken_sps.contains(&sp) || n.optional_treasures_pod.contains(name))
        };
        for i in c.p.items.iter() {
            if !considered(i.name.to_ascii_lowercase().as_str(), i.sp) {
                continue;
            }
            let y = c.sp_pos(i.sp).y;
            if y == 100.0 || y == 125.0 {
                adj_seconds += 12.0;
            }
        }
        for ii in 0..c.p.items.len() {
            let name = c.p.items[ii].name.to_ascii_lowercase();
            let isp = c.p.items[ii].sp;
            if !considered(name.as_str(), isp) {
                continue;
            }
            if c.sp_pos(isp).y >= 30.0 || carry_info(&name).min_carry > 10 {
                continue;
            }
            let d_item = c.dist_to_start(isp);
            if d_item < 170.0 {
                continue;
            }
            let mut worst = 0.0f32;
            for t in c.p.tekis.iter() {
                if t.name.eq_ignore_ascii_case("panmodoki") {
                    let closeness = c.sp_dist(t.sp, isp) / d_item;
                    if closeness < 0.25 {
                        worst = worst.max((1.5 / closeness).min(15.0));
                    }
                }
            }
            adj_seconds += worst;
        }
    }

    out.walking = walking_seconds as f64;
    out.carry = carry_seconds as f64;
    out.hazard = hazard_seconds as f64;
    out.gate = gate_seconds as f64;
    out.optional = optional_seconds as f64;
    out.poko = (j.expected_num_pokos_pod - num_pokos_collected) as f64 / 2.0;
    out.bug = bug_pokos as f64 / 2.0;
    out.adj = adj_seconds as f64;
    out.taken = num_treasures_collected;
    out
}

/// Every take/skip choice considered: all subsets for up to 4 counted treasures, otherwise
/// taking all and skipping each one alone. Taking all comes first.
fn variants(j: &JhawkJudge, layout: &Layout) -> Vec<Parts> {
    let k = counted(&ctx(j, layout)).len();
    let masks: Vec<Vec<bool>> = if k <= 4 {
        (0..1u32 << k).map(|m| (0..k).map(|i| m >> i & 1 == 1).collect()).collect()
    } else {
        std::iter::once(vec![false; k])
            .chain((0..k).map(|i| (0..k).map(|x| x == i).collect()))
            .collect()
    };
    masks.iter().map(|m| parts(j, layout, m)).collect()
}

/// The fitted weights. The poko term keeps weight 1: the runs' times count 2 pokos per second.
struct Model {
    walking: f64,
    carry: f64,
    hazard: f64,
    gate: f64,
    /// optional-treasure credit (runners almost never take them)
    optional: f64,
    /// bug-poko credit
    bug: f64,
    /// Glutton's Kitchen adjustments (high treasures, breadbugs)
    adj: f64,
    /// seconds per collected treasure (cutscene and panel)
    per_treasure: f64,
    /// SH-6: taking the lens when its carry route crosses water
    lens_water: f64,
    /// how sure the runner's choice is: choices are weighted by exp(-score / temp)
    temp: f64,
}

const MODEL: Model = Model {
    walking: 1.22,
    carry: 0.78,
    hazard: 0.60,
    gate: 0.58,
    optional: 0.0,
    bug: 0.0,
    adj: 0.54,
    per_treasure: 10.0,
    lens_water: 100.0,
    temp: 5.0,
};

impl Model {
    fn score(&self, v: &Parts) -> f64 {
        self.walking * v.walking + self.carry * v.carry + self.hazard * v.hazard + self.gate * v.gate + self.optional * v.optional + v.poko
            - self.bug * v.bug
            + self.adj * v.adj
            + self.per_treasure * v.taken as f64
            + if v.lens_cross > 0.0 { self.lens_water } else { 0.0 }
    }
}

pub(super) fn judge(j: &JhawkJudge, layout: &Layout) -> Judge2 {
    let vs = variants(j, layout);
    let m = &MODEL;
    let scores: Vec<f64> = vs.iter().map(|v| m.score(v)).collect();
    let best = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let w: Vec<f64> = scores.iter().map(|s| (-(s - best) / m.temp).exp()).collect();
    let wsum: f64 = w.iter().sum();
    let score = scores.iter().zip(&w).map(|(s, w)| s * w).sum::<f64>() / wsum;
    let choices = vs
        .into_iter()
        .zip(&scores)
        .zip(&w)
        .map(|((v, &s), w)| Judge2Choice {
            skipped: v.skipped,
            score: s,
            p: w / wsum,
        })
        .collect();
    Judge2 { score, choices }
}

// ----------------------------------------------------------------------------
// Water on carry routes
// ----------------------------------------------------------------------------

/// The map units' waterboxes as rectangles in game coordinates: (x1, z1, x2, z2).
fn water_rects(layout: &Layout) -> Vec<(f32, f32, f32, f32)> {
    let mut out = Vec::new();
    for u in layout.units.iter() {
        let cx = (u.x as f32 + u.unit.width as f32 / 2.0) * 170.0;
        let cz = (u.z as f32 + u.unit.height as f32 / 2.0) * 170.0;
        for w in u.unit.waterboxes.iter() {
            out.push((cx + w.p1[0], cz + w.p1[2], cx + w.p2[0], cz + w.p2[2]));
        }
    }
    out
}

fn in_water(rects: &[(f32, f32, f32, f32)], x: f32, z: f32) -> bool {
    rects.iter().any(|&(x1, z1, x2, z2)| x >= x1 && x <= x2 && z >= z1 && z <= z2)
}

/// The route from a point to the ship: its closest waypoint, then back pointers.
fn path_wps(c: &mut Ctx, p: V3) -> Vec<usize> {
    let mut out = Vec::new();
    let Some(mut w) = c.g.closest_wp(p) else { return out };
    loop {
        out.push(w);
        if c.g.wps[w].is_start {
            break;
        }
        match c.g.wps[w].back {
            Some(b) if !out.contains(&b) => w = b,
            _ => break,
        }
    }
    out
}

/// Water on the route from a point to the ship after its first dry stretch (sampled every
/// ~10 units): water that must be crossed, not the pool the object starts in. Also returns the
/// route's waypoints.
fn route_cross(c: &mut Ctx, rects: &[(f32, f32, f32, f32)], p: V3) -> (f32, Vec<usize>) {
    let wps = path_wps(c, p);
    c.g.dist_to_start(p); // diagnostic only (see the module notes)
    let (mut cross, mut dry) = (0.0f32, false);
    let mut prev = p;
    for &w in wps.iter() {
        let q = c.g.wps[w].pos;
        let len = dist(prev, q);
        let n = ((len / 10.0).ceil() as usize).max(1);
        for i in 0..n {
            let t = (i as f32 + 0.5) / n as f32;
            if !rects.is_empty() && in_water(rects, prev.x + (q.x - prev.x) * t, prev.z + (q.z - prev.z) * t) {
                if dry {
                    cross += len / n as f32;
                }
            } else {
                dry = true;
            }
        }
        prev = q;
    }
    (cross, wps)
}

/// Gates whose door is on this route (the gate's closest waypoint is on it).
fn gates_on(c: &mut Ctx, wps: &[usize]) -> Vec<usize> {
    let set: HashSet<usize> = wps.iter().copied().collect();
    let mut out = Vec::new();
    for g in 0..c.p.gates.len() {
        let p = c.sp_pos(c.p.gates[g].sp);
        if let Some(w) = c.g.closest_wp(p) {
            if set.contains(&w) {
                out.push(g);
            }
        }
    }
    out
}
