//! Generates the same seeds with CaveGen3 and Caveripper (the reference) and checks the
//! layouts are identical: units, scores, door scores, and every spawned object with its
//! exact position, in the same order.
//!
//! Set `VS_REFERENCE_SEEDS` to change how many seeds each floor gets (default 300).

use caveripper::{
    assets::{AssetManager, fs_asset_manager::FsAssetManager},
    layout::{Layout as RefLayout, SpawnObject as RefObject},
    sublevel::Sublevel as RefSublevel,
};
use cavegen3::{GameData, Layout, SpawnObject};
use rayon::prelude::*;

fn describe(so: &SpawnObject, pos: cavegen3::point::Point) -> String {
    let what = match so {
        SpawnObject::Teki(t, _) => format!("teki {} g{}", t.internal_name, t.group),
        SpawnObject::CapTeki(t, n) => format!("capteki {} x{n}", t.internal_name),
        SpawnObject::Item(i) => format!("item {}", i.internal_name),
        SpawnObject::Gate(g, r) => format!("gate {} r{r}", g.health),
        SpawnObject::Hole(p) => format!("hole {p}"),
        SpawnObject::Geyser(c) => format!("geyser {c}"),
        SpawnObject::Ship => "ship".into(),
        SpawnObject::Onion(c) => format!("onion {c}"),
    };
    format!("{what} @ {:?}", pos.0)
}

fn describe_ref(so: &RefObject, pos: caveripper::point::Point<3, f32>) -> String {
    let what = match so {
        RefObject::Teki(t, _) => format!("teki {} g{}", t.internal_name, t.group),
        RefObject::CapTeki(t, n) => format!("capteki {} x{n}", t.internal_name),
        RefObject::Item(i) => format!("item {}", i.internal_name),
        RefObject::Gate(g, r) => format!("gate {} r{r}", g.health),
        RefObject::Hole(p) => format!("hole {p}"),
        RefObject::Geyser(c) => format!("geyser {c}"),
        RefObject::Ship => "ship".into(),
        RefObject::Onion(c) => format!("onion {c}"),
    };
    format!("{what} @ {:?}", pos.0)
}

fn summarize(layout: &Layout) -> Vec<String> {
    let mut out: Vec<String> = layout
        .units
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let doors: Vec<i32> = layout.unit_doors(i).iter().map(|d| d.door_score).collect();
            format!(
                "unit {} r{} {},{} score {} teki {} doors {doors:?}",
                u.unit.unit_folder_name, u.unit.rotation, u.x, u.z, u.total_score, u.teki_score
            )
        })
        .collect();
    out.extend(layout.spawn_objects().map(|(so, pos)| describe(so, pos)));
    out
}

fn summarize_ref(layout: &RefLayout) -> Vec<String> {
    let mut out: Vec<String> = layout
        .map_units
        .iter()
        .map(|u| {
            let doors: Vec<i32> = u.doors.iter().map(|d| d.borrow().door_score).collect();
            format!(
                "unit {} r{} {},{} score {} teki {} doors {doors:?}",
                u.unit.unit_folder_name, u.unit.rotation, u.x, u.z, u.total_score, u.teki_score
            )
        })
        .collect();
    out.extend(layout.get_spawn_objects().map(|(so, pos)| describe_ref(so, pos)));
    out
}

/// Seeds 0..n plus as many spread over the whole seed space.
fn seeds(n: u32) -> Vec<u32> {
    let mut seeds: Vec<u32> = (0..n).collect();
    let mut x = 0x1234_5678u32;
    for _ in 0..n {
        x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        seeds.push(x);
    }
    seeds
}

#[test]
fn matches_reference() {
    let n: u32 = std::env::var("VS_REFERENCE_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(300);
    let data = GameData::bundled().unwrap();
    let mgr = FsAssetManager::init().unwrap();
    let seeds = seeds(n);

    let mut floors = Vec::new();
    // New Year 1.38's Elemental River and Season Den use a unit file with a unit missing its `}`
    // (1_units_big2_rocket_desert.txt, 1_MAT_boss_volcano.txt), which the game ignores. Caveripper can't parse Lands of Torture's Bulblax Kingdom (a CapInfo missing its `{`), and it drops the
    // two spawn points missing their `{` in room_dangomushi_tile, which only Cavern of Chaos
    // floor 10 uses. cavegen_expect checks those against CaveGen instead.
    let skip = |c: &cavegen3::data::CaveConfig| {
        (c.game == "torture" && c.full_name == "Bulblax Kingdom")
            || (c.game == "newyear" && (c.full_name == "Elemental River" || c.full_name == "Season Den"))
    };
    for cfg in data.caves().iter().filter(|c| data.has_game(&c.game) && !skip(c)) {
        let ref_cfg = mgr.cave_cfg.iter().find(|c| c.caveinfo_filename == cfg.caveinfo_filename && c.game == cfg.game).unwrap();
        for (i, ci) in data.floors(cfg).unwrap().iter().enumerate() {
            if cfg.game == "torture" && cfg.full_name == "Cavern of Chaos" && i + 1 == 10 {
                continue;
            }
            let ref_ci = mgr.load_caveinfo(&RefSublevel::from_cfg(ref_cfg, i + 1)).unwrap();
            floors.push((ci, ref_ci));
        }
    }

    let failures: Vec<String> = floors
        .par_iter()
        .filter_map(|(ci, ref_ci)| {
            for &seed in seeds.iter() {
                let ours = summarize(&Layout::generate(seed, ci));
                let theirs = summarize_ref(&RefLayout::generate(seed, ref_ci));
                if ours != theirs {
                    let i = ours.iter().zip(&theirs).position(|(a, b)| a != b).unwrap_or(ours.len().min(theirs.len()));
                    return Some(format!(
                        "{} seed {seed:#010X}: line {i} of {}/{}\n  ours:   {}\n  theirs: {}",
                        ci.name(),
                        ours.len(),
                        theirs.len(),
                        ours.get(i).map_or("<none>", |s| s),
                        theirs.get(i).map_or("<none>", |s| s)
                    ));
                }
            }
            None
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} floors differ (first difference per floor):\n{}",
        failures.len(),
        floors.len(),
        failures.join("\n")
    );
}
