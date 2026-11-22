//! Special-purpose searches that the query language can't express.
//!
//! Ported from Caveripper's `query::special` (MIT, https://github.com/mayabyte/caveripper).

use super::Query;
use crate::{
    caveinfo::TekiInfo,
    data::{GameData, Sublevel},
    layout::{Layout, SpawnObject},
    rng::Generator,
};

/// Matches seeds where this seed and the next `num_consecutive - 1` seeds generate the same
/// map units and treasure positions.
pub struct ConsecutiveIdenticalSeedsQuery {
    pub sublevel: Sublevel,
    pub num_consecutive: u32,
}

impl Query for ConsecutiveIdenticalSeedsQuery {
    fn matches_with(&self, generator: Generator, seed: u64, data: &GameData) -> bool {
        let caveinfo = data.caveinfo(&self.sublevel).expect("sublevel has no caveinfo");
        let last = match generator {
            Generator::Gc => (seed as u32).saturating_add(self.num_consecutive) as u64,
            Generator::Switch => seed.saturating_add(self.num_consecutive as u64),
        };
        let layouts: Vec<Layout> = (seed..last).map(|s| Layout::generate_with(generator, s, caveinfo)).collect();
        let units = |l: &Layout| l.units.iter().map(|u| (u.unit.unit_folder_name.clone(), u.unit.rotation, u.x, u.z)).collect::<Vec<_>>();
        let treasures = |l: &Layout| {
            l.spawn_objects()
                .filter_map(|(so, pos)| match so {
                    SpawnObject::Item(info) => Some((pos.0, info.internal_name.clone())),
                    SpawnObject::Teki(TekiInfo { carrying: Some(carried), .. }, _) => Some((pos.0, carried.clone())),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let first = &layouts[0];
        let (first_units, first_treasures) = (units(first), treasures(first));
        layouts[1..].iter().all(|l| units(l) == first_units && treasures(l) == first_treasures)
    }
}
