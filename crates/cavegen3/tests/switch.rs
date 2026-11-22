//! The Switch generator ([`Generator::Switch`]): every vanilla sublevel generates with it.
//!
//! There are no reference Switch layouts to compare against yet, so this checks that generation
//! completes (no panics from out-of-range draws, which 31-bit `rand()` makes possible) and that
//! `generate_with(Gc, ..)` is exactly `generate`.

use cavegen3::{GameData, Generator, Layout};
use rayon::prelude::*;

/// Spreads 0, 1, 2, ... over 64-bit seeds (SplitMix64's finalizer).
fn mix64(i: u64) -> u64 {
    let mut x = i.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn signature(layout: &Layout) -> Vec<(String, i32, i32, u16)> {
    layout.units.iter().map(|u| (u.unit.unit_folder_name.clone(), u.x, u.z, u.unit.rotation)).collect()
}

#[test]
fn every_vanilla_sublevel_generates_with_the_switch_generator() {
    let data = GameData::bundled().expect("bundled game data");
    let caves: Vec<_> = data.caves().iter().filter(|c| c.game == "pikmin2").collect();
    assert!(caves.len() >= 44, "expected the 14 story caves and 30 challenge caves");
    let differ = caves
        .par_iter()
        .map(|cfg| {
            let floors = data.floors(cfg).unwrap();
            let mut differ = 0;
            for ci in floors {
                for i in 0..200u64 {
                    let seed = mix64(i ^ (ci.floor_num as u64) << 32);
                    let sw = Layout::generate_with(Generator::Switch, seed, ci);
                    assert_eq!(sw.seed, seed);
                    assert_eq!(sw.generator, Generator::Switch);
                    let gc = Layout::generate(seed as u32, ci);
                    if signature(&sw) != signature(&gc) {
                        differ += 1;
                    }
                }
            }
            (differ, floors.len() * 200)
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    // The two generators give unrelated layouts (sublevels with a fixed map, or one unit, match).
    assert!(differ.0 * 10 > differ.1 * 8, "{differ:?}"); // about 89%
}

#[test]
fn generate_with_gc_is_generate() {
    let data = GameData::bundled().expect("bundled game data");
    let sublevel = data.sublevel("SCx-6").unwrap();
    let ci = data.caveinfo(&sublevel).unwrap();
    for seed in [0u32, 0x1234, 0xDEAD_BEEF, u32::MAX] {
        let a = Layout::generate(seed, ci);
        // High bits are ignored on GC.
        let b = Layout::generate_with(Generator::Gc, seed as u64 | 0xABCD << 40, ci);
        assert_eq!(signature(&a), signature(&b));
        assert_eq!(b.seed, seed as u64);
    }
}
