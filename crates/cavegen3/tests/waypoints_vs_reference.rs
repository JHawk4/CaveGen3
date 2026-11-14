//! Checks the waypoint graph and carry paths match Caveripper's exactly.

use caveripper::{
    assets::{AssetManager, fs_asset_manager::FsAssetManager},
    layout::{Layout as RefLayout, waypoint::get_path_to_goal as ref_path_to_goal},
    sublevel::Sublevel as RefSublevel,
};
use cavegen3::{GameData, Layout, waypoint::get_path_to_goal};
use rayon::prelude::*;

#[test]
fn carry_paths_match_reference() {
    let data = GameData::bundled().unwrap();
    let mgr = FsAssetManager::init().unwrap();
    let mut floors = Vec::new();
    for cfg in data.caves().iter().filter(|c| c.game == "pikmin2") {
        let ref_cfg = mgr.cave_cfg.iter().find(|c| c.caveinfo_filename == cfg.caveinfo_filename && c.game == cfg.game).unwrap();
        for (i, ci) in data.floors(cfg).unwrap().iter().enumerate() {
            floors.push((ci, mgr.load_caveinfo(&RefSublevel::from_cfg(ref_cfg, i + 1)).unwrap()));
        }
    }

    let failures: Vec<String> = floors
        .par_iter()
        .filter_map(|(ci, ref_ci)| {
            for seed in 0..15u32 {
                let ours = Layout::generate(seed, ci);
                let theirs = RefLayout::generate(seed, ref_ci);
                let nodes: Vec<_> = ours.waypoint_graph().iter().map(|n| (n.pos.0, n.dist_to_start)).collect();
                let ref_nodes: Vec<_> = theirs.waypoint_graph().iter().map(|n| (n.pos.0, n.dist_to_start)).collect();
                if nodes != ref_nodes {
                    return Some(format!("{} seed {seed}: waypoint graph differs", ci.name()));
                }
                let positions: Vec<_> = ours.spawn_objects().map(|(_, p)| p).collect();
                for pos in positions {
                    let ref_pos = caveripper::point::Point(pos.0);
                    let chain: Vec<_> = ours.waypoint_graph().carry_path_wps_pos(pos).map(|p| p.0).collect();
                    let ref_chain: Vec<_> = theirs.waypoint_graph().carry_path_wps_pos(ref_pos).map(|p| p.0).collect();
                    if chain != ref_chain {
                        return Some(format!("{} seed {seed}: carry path from {:?} differs", ci.name(), pos.0));
                    }
                    let path = get_path_to_goal(pos, 1.0, 10001, ours.waypoint_graph().carry_path_wps_nodes(pos).collect());
                    let ref_path = ref_path_to_goal(ref_pos, 1.0, 10001, theirs.waypoint_graph().carry_path_wps_nodes(ref_pos).collect());
                    if path.iter().map(|p| p.0).ne(ref_path.iter().map(|p| p.0)) {
                        return Some(format!("{} seed {seed}: smoothed path from {:?} differs", ci.name(), pos.0));
                    }
                }
            }
            None
        })
        .collect();
    assert!(failures.is_empty(), "{} of {} floors differ:\n{}", failures.len(), floors.len(), failures.join("\n"));
}
