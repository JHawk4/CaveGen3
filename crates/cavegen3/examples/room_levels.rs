//! Checks `room_odds` against real seeds: how many seeds reach each splitting level.
//! `room_levels SR-7 [count (hex)] [gc|switch]` (GC seeds count up from 0, Switch seeds are mixed).

use std::sync::Mutex;

use cavegen3::{GameData, Generator, generate::failure_levels_reached, rng::Rng};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("SR-7");
    let n = args.get(2).map(|s| u64::from_str_radix(s, 16).unwrap()).unwrap_or(1 << 24);
    let generator: Generator = args.get(3).map(|s| s.parse().unwrap()).unwrap_or_default();
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let counts = Mutex::new(vec![0u64; 16]);
    (0..n.div_ceil(1 << 16)).into_par_iter().for_each(|c| {
        let mut local = vec![0u64; 16];
        for i in (c << 16)..((c + 1) << 16).min(n) {
            let seed = match generator {
                Generator::Gc => i,
                Generator::Switch => i.wrapping_mul(0x9E3779B97F4A7C15) ^ 0x5EED,
            };
            for l in 0..failure_levels_reached(Rng::with(generator, seed), ci) {
                local[l] += 1;
            }
        }
        let mut c = counts.lock().unwrap();
        for (a, b) in c.iter_mut().zip(&local) {
            *a += b;
        }
    });
    let c = counts.lock().unwrap();
    println!("{sub} ({generator:?}), {n} seeds: seeds reaching each level, and the share of the level before");
    for l in 0..c.len() {
        if c[l] > 0 {
            let before = if l == 0 { n } else { c[l - 1] };
            println!("  level {l}: {:>10}  {:.4}", c[l], c[l] as f64 / before as f64);
        }
    }
}
