//! Scans seeds for maps that close before all their rooms are placed (Shower Room 7's missing
//! boss): `cargo run --release -p cavegen3 --example room_scan -- SR-7 [start] [end] [gc|switch]`
//! (hex; default all 2^31 GC seeds). Prints each such seed, and how many placement steps the
//! last room took over all seeds.

use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use cavegen3::{GameData, Generator, generate::rooms_placed, rng::Rng};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("SR-7");
    let hex = |i: usize, d: u64| {
        args.get(i)
            .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
            .unwrap_or(d)
    };
    let (start, end) = (hex(2, 0), hex(3, 1 << 31));
    let generator: Generator = args.get(4).map(|s| s.parse().unwrap()).unwrap_or_default();
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let want = ci.num_rooms;
    let t0 = std::time::Instant::now();
    let done = AtomicU64::new(0);
    let hist: Mutex<Vec<u64>> = Mutex::new(vec![0; 502]);
    let chunk = 1u64 << 20;
    let chunks: Vec<u64> = (start..end).step_by(chunk as usize).collect();
    chunks.par_iter().for_each(|&c0| {
        let mut local = vec![0u64; 502];
        for seed in c0..(c0 + chunk).min(end) {
            let (rooms, steps) = rooms_placed(Rng::with(generator, seed), ci);
            local[steps as usize] += 1;
            if rooms < want {
                println!("FOUND {sub} seed {seed:#X}: {rooms} of {want} rooms after {steps} steps");
            }
        }
        let mut h = hist.lock().unwrap();
        for (a, b) in h.iter_mut().zip(&local) {
            *a += b;
        }
        let n = done.fetch_add((c0 + chunk).min(end) - c0, Ordering::Relaxed) + chunk;
        if (n / chunk) % 64 == 0 {
            let s = t0.elapsed().as_secs_f64();
            eprintln!("{n} of {} seeds, {s:.0} s, {:.0}/s", end - start, n as f64 / s);
        }
    });
    let s = t0.elapsed().as_secs_f64();
    println!("Scanned {} seeds in {s:.0} s ({:.0}/s)", end - start, (end - start) as f64 / s);
    println!("Steps until the last room was placed (or the map closed):");
    for (k, n) in hist.lock().unwrap().iter().enumerate() {
        if *n > 0 {
            println!("  {k:>3} {n}");
        }
    }
}
