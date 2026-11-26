//! How likely each spot was to get the exit (the hole, or the geyser on a final floor), seen from
//! the finished layout: for Challenge Mode, where the seed isn't known but the map and what's on
//! it are.
//!
//! The game places the exit right after the pod, before any teki or treasure (`place_hole`): in
//! an empty alcove (group 4) or dead end (group 9), picked at random with weight sqrt(unit score)
//! + 10 in Challenge Mode (its `hole_score`). Spots that end up holding something else can't
//! have had it, and teki keep away from it afterwards: hard teki (group 1) 200 units, special
//! teki (group 8) 150. The spots left share the chance in proportion to their weights. In story
//! mode the highest score always wins, so there are no odds to give.

use std::collections::BTreeMap;

use crate::{Layout, SpawnObject};

fn is_exit(so: &SpawnObject) -> bool {
    matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_))
}

/// The chance of the exit being at each spawn point of `layout` (only spots with a chance are
/// listed), given everything else on the map; empty outside Challenge Mode. See the module docs.
pub fn exit_odds(layout: &Layout) -> BTreeMap<u32, f64> {
    if !layout.caveinfo.is_challenge_mode() {
        return BTreeMap::new();
    }
    let sps = &layout.spawnpoints;
    let mut cands: Vec<(usize, f64)> = sps
        .iter()
        .enumerate()
        .filter(|(_, sp)| matches!(sp.group(), 4 | 9) && sp.hole_score > 0)
        .map(|(i, sp)| (i, sp.hole_score as f64))
        .collect();
    // Teki that keep away from the exit: (where, how far).
    let keep_away: Vec<_> = sps
        .iter()
        .filter(|sp| sp.contains.iter().any(|so| matches!(so, SpawnObject::Teki(..))))
        .filter_map(|sp| match sp.group() {
            1 => Some((sp.pos, 200.0)),
            8 => Some((sp.pos, 150.0)),
            _ => None,
        })
        .collect();
    cands.retain(|&(i, _)| {
        let sp = &sps[i];
        sp.contains.iter().all(is_exit) && keep_away.iter().all(|(pos, gap)| pos.p2_dist(&sp.pos) >= *gap)
    });
    let total: f64 = cands.iter().map(|c| c.1).sum();
    if total <= 0.0 {
        return BTreeMap::new();
    }
    cands.into_iter().map(|(i, w)| (i as u32, w / total)).collect()
}
