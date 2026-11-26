//! Where the exit (the hole, or the geyser on a final floor) is likely to be when the seed isn't
//! known (Challenge Mode), worked out from the layout itself with no other layouts generated.
//!
//! The game places the hole right after the pod, before any teki or treasure (`place_hole`):
//! among the empty alcoves (group 4, except those within 150 of the pod in its own unit) and dead
//! ends (group 9) of rooms and caps, or of hallways if there are none. In Challenge Mode it picks
//! one at random with weight sqrt(unit score) + 10; in story mode the highest score wins. So a
//! spot's chance is its weight over the total. What the runner has seen ([`Fact`]s about spots)
//! rules spots out, and the rest share the chance in proportion to their weights.
//!
//! Teki keep away from the exit afterwards (hard teki 200 units, special teki 150), so a teki
//! seen in a hard or special teki spot also rules out the spots near it.

use std::collections::BTreeMap;

use crate::{Layout, SpawnObject};

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
    /// Something other than the exit is there (a teki, treasure, plant...).
    Occupied,
    /// Nothing the comedown shows is there (it may hold a teki, which the comedown hides).
    NothingVisible,
    /// The exit is here.
    Exit,
    /// The exit isn't here.
    NoExit,
}

impl Fact {
    /// The one-letter code used in URLs and commands.
    pub fn code(self) -> char {
        match self {
            Fact::Empty => 'e',
            Fact::Occupied => 'o',
            Fact::NothingVisible => 'v',
            Fact::Exit => 'h',
            Fact::NoExit => 'n',
        }
    }

    pub fn from_code(c: char) -> Option<Fact> {
        Some(match c {
            'e' => Fact::Empty,
            'o' => Fact::Occupied,
            'v' => Fact::NothingVisible,
            'h' => Fact::Exit,
            'n' => Fact::NoExit,
            _ => return None,
        })
    }

    /// Whether a spawn point holding `contains` agrees with this fact.
    pub fn holds(self, contains: &[SpawnObject]) -> bool {
        let is_exit = |so: &SpawnObject| matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_));
        match self {
            Fact::Empty => contains.is_empty(),
            Fact::Occupied => contains.iter().any(|so| !is_exit(so)),
            Fact::NothingVisible => !contains.iter().any(comedown_visible),
            Fact::Exit => contains.iter().any(is_exit),
            Fact::NoExit => !contains.iter().any(is_exit),
        }
    }

    /// Parses facts about spawn points: "12e,40h" (spawn point index, then the code), or "-" or
    /// "" for none.
    pub fn parse_list(text: &str) -> Result<Vec<(u32, Fact)>, String> {
        let text = text.trim();
        if text.is_empty() || text == "-" {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for p in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let c = p.chars().last().unwrap();
            let fact = Fact::from_code(c).ok_or_else(|| format!("bad fact {p:?}"))?;
            let index = p[..p.len() - c.len_utf8()].parse().map_err(|_| format!("bad fact {p:?}"))?;
            out.push((index, fact));
        }
        Ok(out)
    }
}

/// The exit `layout` has: the geyser on a final floor, else the hole (a story floor with both
/// gets the hole's odds).
pub fn exit_of(layout: &Layout) -> Option<u32> {
    if layout.caveinfo.is_final_floor { layout.geyser } else { layout.hole }
}

/// Where the exit can go in `layout` and each spot's weight, as `place_hole` picks it (see the
/// module docs): sqrt(unit score) + 10 in Challenge Mode; in story mode 1 for the highest score
/// and 0 for the rest.
///
/// `layout` must be generated with `exits_only` ([`generate_partly`](crate::generate::generate_partly)):
/// the unit scores are then still the ones the game had when it placed the exit (teki raise them).
pub fn exit_candidates(layout: &Layout) -> Vec<(u32, f64)> {
    let scores = candidate_scores(layout);
    pick_weights(layout, &scores.iter().collect::<Vec<_>>())
}

/// The weights `place_hole` gives `cands` (spawn point, score): the score itself in Challenge
/// Mode, else 1 for the highest score and 0 for the rest.
fn pick_weights(layout: &Layout, cands: &[&(u32, i32)]) -> Vec<(u32, f64)> {
    if layout.caveinfo.is_challenge_mode() {
        cands.iter().map(|&&(i, w)| (i, w.max(0) as f64)).collect()
    } else {
        let best = cands.iter().map(|c| c.1).max().unwrap_or(0);
        cands.iter().map(|&&(i, w)| (i, if w == best { 1.0 } else { 0.0 })).collect()
    }
}

/// The exit candidates of `layout` and their scores (see [`exit_candidates`]).
fn candidate_scores(layout: &Layout) -> Vec<(u32, i32)> {
    use crate::{caveinfo::RoomType, math};
    let challenge = layout.caveinfo.is_challenge_mode();
    let Some(ship) = layout.ship else { return Vec::new() };
    let ship_sp = &layout.spawnpoints[ship as usize];
    let mut out: Vec<(u32, i32)> = Vec::new();
    for room_type in [RoomType::Room, RoomType::DeadEnd, RoomType::Hallway] {
        if room_type == RoomType::Hallway && !out.is_empty() {
            continue;
        }
        for (ui, unit) in layout.units.iter().enumerate() {
            if unit.unit.room_type != room_type {
                continue;
            }
            let score = if challenge {
                let root = if unit.total_score > 0 {
                    math::sqrt(unit.total_score as f32)
                } else {
                    0.0
                };
                root as i32 + 10
            } else {
                unit.total_score
            };
            for si in unit.spawnpoint_indices() {
                let sp = &layout.spawnpoints[si];
                // The hole and geyser are the only things placed yet; their spots were empty.
                if sp.contains.iter().any(|so| matches!(so, SpawnObject::Ship)) {
                    continue;
                }
                let g = sp.group();
                let usable = (g == 4 && (ui as u32 != ship_sp.unit || ship_sp.pos.p2_dist(&sp.pos) >= 150.0)) || g == 9;
                if usable {
                    out.push((si as u32, score));
                }
            }
        }
    }
    out
}

/// Whether the exit could be at spawn point `i` of `layout` given the facts: not at a spot seen
/// empty or holding something else, nor near a seen hard teki (200) or special teki (150), which
/// keep that far from it; and only at the spot marked as the exit, if one is.
fn allowed(layout: &Layout, facts: &[(u32, Fact)], i: u32) -> bool {
    let pos = layout.spawnpoints[i as usize].pos;
    facts.iter().all(|&(fi, fact)| {
        let here = fi == i;
        match fact {
            Fact::Exit => here,
            Fact::NoExit => !here,
            Fact::Empty | Fact::Occupied | Fact::NothingVisible => {
                if here {
                    return false;
                }
                // A seen teki in a hard (1) or special (8) teki spot.
                let Some(sp) = layout.spawnpoints.get(fi as usize) else { return true };
                let gap = match (fact, sp.group()) {
                    (Fact::Occupied, 1) => 200.0,
                    (Fact::Occupied, 8) => 150.0,
                    _ => return true,
                };
                sp.pos.p2_dist(&pos) >= gap
            }
        }
    })
}

/// The chance of the exit being at each spawn point of `layout`, given `facts`: each candidate's
/// weight over the total, after dropping the spots the facts rule out. Empty when they rule out
/// every spot (or the floor has no exit). `layout` must be generated with `exits_only` (see
/// [`exit_candidates`]).
pub fn odds(layout: &Layout, facts: &[(u32, Fact)]) -> BTreeMap<u32, f64> {
    let cands: Vec<(u32, f64)> = exit_candidates(layout)
        .into_iter()
        .filter(|&(i, w)| w > 0.0 && allowed(layout, facts, i))
        .collect();
    let total: f64 = cands.iter().map(|c| c.1).sum();
    if total <= 0.0 {
        return BTreeMap::new();
    }
    cands.into_iter().map(|(i, w)| (i, w / total)).collect()
}
