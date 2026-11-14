//! Times CaveGen3's generator against Caveripper's (the reference) on every bundled floor.
//!
//! `cargo run --release -p cavegen3 --example bench [seeds_per_floor] [repeats]`
//!
//! For each floor both generators run the same seeds, alternating, and the fastest of
//! `repeats` runs is kept, so background load affects both about equally.

use std::{hint::black_box, time::Instant};

use caveripper::{
    assets::{AssetManager, fs_asset_manager::FsAssetManager},
    layout::Layout as RefLayout,
    sublevel::Sublevel as RefSublevel,
};
use cavegen3::{GameData, Layout};

fn time_per_layout(n: u32, f: impl Fn(u32)) -> f64 {
    let start = Instant::now();
    for seed in 0..n {
        f(seed);
    }
    start.elapsed().as_secs_f64() * 1e6 / n as f64
}

fn main() {
    let n: u32 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(500);
    let repeats: u32 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(3);
    let data = GameData::bundled().unwrap();
    let mgr = FsAssetManager::init().unwrap();

    let mut rows = Vec::new();
    for cfg in data.caves().iter().filter(|c| data.has_game(&c.game)) {
        let ref_cfg = mgr.cave_cfg.iter().find(|c| c.full_name == cfg.full_name && c.game == cfg.game).unwrap();
        for (i, ci) in data.floors(cfg).unwrap().iter().enumerate() {
            let ref_ci = mgr.load_caveinfo(&RefSublevel::from_cfg(ref_cfg, i + 1)).unwrap();
            let (mut ours, mut theirs) = (f64::INFINITY, f64::INFINITY);
            for _ in 0..repeats {
                ours = ours.min(time_per_layout(n, |s| {
                    black_box(Layout::generate(s, ci));
                }));
                theirs = theirs.min(time_per_layout(n, |s| {
                    black_box(RefLayout::generate(s, ref_ci));
                }));
            }
            rows.push((ci.name(), ours, theirs));
        }
    }

    let mean = |f: fn(&(String, f64, f64)) -> f64| rows.iter().map(f).sum::<f64>() / rows.len() as f64;
    let (ours, theirs) = (mean(|r| r.1), mean(|r| r.2));
    println!("{} floors x {n} seeds, best of {repeats}", rows.len());
    println!("CaveGen3:   {ours:8.2} us/layout ({:.0} layouts/s on one thread)", 1e6 / ours);
    println!("Caveripper: {theirs:8.2} us/layout ({:.0} layouts/s on one thread)", 1e6 / theirs);
    println!("Speedup:    {:.2}x average", theirs / ours);
    let vanilla: Vec<_> = rows.iter().filter(|r| r.0 != "colossal1").collect();
    let (vo, vt) = (
        vanilla.iter().map(|r| r.1).sum::<f64>() / vanilla.len() as f64,
        vanilla.iter().map(|r| r.2).sum::<f64>() / vanilla.len() as f64,
    );
    let mut ratios: Vec<f64> = vanilla.iter().map(|r| r.2 / r.1).collect();
    ratios.sort_by(f64::total_cmp);
    println!(
        "Pikmin 2 floors only: {vo:.2} vs {vt:.2} us/layout, {:.2}x average, {:.2}x median per floor",
        vt / vo,
        ratios[ratios.len() / 2]
    );
    rows.sort_by(|a, b| b.2.total_cmp(&a.2));
    println!("Slowest floors (CaveGen3 / Caveripper, us):");
    for (name, o, t) in rows.iter().take(8) {
        println!("  {name:10} {o:8.1} / {t:8.1}  ({:.1}x)", t / o);
    }
}
