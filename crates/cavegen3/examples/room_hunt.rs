//! Hunts random seeds for maps that close before all their rooms are placed (Shower Room 7's
//! missing boss), until it finds enough: `room_hunt SR-7 [gc|switch] [how many] [first batch]`.
//! Seeds are a hash of a counter, so they spread over the whole seed space (all 2^64 on Switch).
//! Each batch is 2^28 seeds; after each it prints the next batch, to resume from there.

use std::{
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
};

use cavegen3::{GameData, Generator, generate::rooms_placed, rng::Rng};
use rayon::prelude::*;

fn mix(i: u64) -> u64 {
    // splitmix64 of the counter
    let mut z = i.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("SR-7");
    let generator: Generator = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(Generator::Switch);
    let want: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(3);
    let mut batch: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(0);
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let mask = if generator == Generator::Gc { 0xFFFF_FFFF } else { u64::MAX };
    let found = AtomicU64::new(0);
    let t0 = std::time::Instant::now();
    let mut tested = 0u64;
    while found.load(Ordering::Relaxed) < want {
        let first = batch << 28;
        (0..256u64).into_par_iter().for_each(|c| {
            for i in first + (c << 20)..first + ((c + 1) << 20) {
                let seed = mix(i) & mask;
                let (rooms, steps) = rooms_placed(Rng::with(generator, seed), ci);
                if rooms < ci.num_rooms {
                    found.fetch_add(1, Ordering::Relaxed);
                    println!("FOUND {sub} {generator:?} seed {seed:016X} (counter {i}): {rooms} of {} rooms after {steps} steps", ci.num_rooms);
                    std::io::stdout().flush().ok();
                }
            }
        });
        tested += 1 << 28;
        batch += 1;
        let s = t0.elapsed().as_secs_f64();
        eprintln!("next batch {batch}: {tested} seeds this run, {} found, {s:.0} s, {:.0}/s", found.load(Ordering::Relaxed), tested as f64 / s);
    }
}
