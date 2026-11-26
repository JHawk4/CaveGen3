//! Hunts random seeds for maps that close with only the start room and are short of a treasure
//! (the way WFG-4 or SR-5 can be), until it finds enough:
//! `closure_hunt SUBLEVEL [gc|switch] [how many] [first batch] [threads]`.
//!
//! Seeds are a hash of a counter, so they spread over the whole seed space (all 2^64 on Switch).
//! Each batch is 2^28 seeds; after each it prints the next batch, to resume from there. Most
//! seeds are ruled out by [`ClosureFilter`]'s one-multiply test of the first corridor roll; the
//! rest are generated until a second room is placed or the map closes. Closed maps are generated
//! in full and their treasures counted.

use std::{
    io::Write,
    sync::atomic::{AtomicU64, Ordering},
};

use cavegen3::{
    GameData, Generator, Layout,
    generate::{ClosureFilter, closes_with_rooms},
    layout::SpawnObject,
    rng::Rng,
};
use rayon::prelude::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn mix(i: u64) -> u64 {
    // splitmix64 of the counter
    let mut z = i.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("WFG-4");
    let generator: Generator = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(Generator::Switch);
    let want: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(1);
    let mut batch: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(0);
    let threads: usize = args.get(5).map(|s| s.parse().unwrap()).unwrap_or(0);
    if threads > 0 {
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
    }
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let filter = ClosureFilter::new(ci, generator);
    let mode = match &filter {
        Some(f) if f.checks_second_step() => "filter on the first two steps",
        Some(_) => "filter on the first step",
        None => "no filter",
    };
    eprintln!("{sub} {generator:?}: {mode}");
    let mask = if generator == Generator::Gc { 0xFFFF_FFFF } else { u64::MAX };
    let found = AtomicU64::new(0);
    let closed = AtomicU64::new(0);
    let t0 = std::time::Instant::now();
    let mut tested = 0u64;
    while found.load(Ordering::Relaxed) < want {
        let first = batch << 28;
        (0..256u64).into_par_iter().for_each(|c| {
            for i in first + (c << 20)..first + ((c + 1) << 20) {
                let seed = mix(i) & mask;
                if filter.as_ref().is_some_and(|f| f.rejects(seed)) || !closes_with_rooms(Rng::with(generator, seed), ci, 1) {
                    continue;
                }
                closed.fetch_add(1, Ordering::Relaxed);
                let layout = Layout::generate_with(generator, seed, ci);
                let treasures: usize = layout
                    .spawnpoints
                    .iter()
                    .map(|sp| sp.contains.iter().filter(|so| matches!(so, SpawnObject::Item(_))).count())
                    .sum();
                let short = (treasures as u32) < ci.max_treasures;
                if short {
                    found.fetch_add(1, Ordering::Relaxed);
                }
                println!(
                    "{} {sub} {generator:?} seed {seed:016X} (counter {i}): closed with 1 room, {treasures} of {} treasures",
                    if short { "FOUND" } else { "CLOSED" },
                    ci.max_treasures
                );
                std::io::stdout().flush().ok();
            }
        });
        tested += 1 << 28;
        batch += 1;
        let s = t0.elapsed().as_secs_f64();
        eprintln!(
            "next batch {batch}: {tested} seeds this run, {} closed, {} found, {s:.0} s, {:.0}/s",
            closed.load(Ordering::Relaxed),
            found.load(Ordering::Relaxed),
            tested as f64 / s
        );
    }
}
