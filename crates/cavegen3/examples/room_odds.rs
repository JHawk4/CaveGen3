//! Estimates the chance a sublevel's map closes before all its rooms are placed (Shower Room 7's
//! missing boss), by splitting on how close the map is to closing: `room_odds SR-7 [gc|switch] [particles] [runs]`.

use cavegen3::{GameData, Generator, generate::room_failure_odds};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("SR-7");
    let generator: Generator = args.get(2).map(|s| s.parse().unwrap()).unwrap_or_default();
    let particles: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(100_000);
    let runs: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(16);
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let t0 = std::time::Instant::now();
    let results: Vec<(f64, Vec<f64>)> = (0..runs)
        .into_par_iter()
        .map(|r| room_failure_odds(ci, generator, particles, 0x5EED_0000 + r))
        .collect();
    let ps: Vec<f64> = results.iter().map(|r| r.0).collect();
    let mean = ps.iter().sum::<f64>() / runs as f64;
    let sd = (ps.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (runs - 1).max(1) as f64).sqrt();
    let se = sd / (runs as f64).sqrt();
    println!(
        "{sub} ({generator:?}), {runs} runs of {particles} maps, {:.0} s",
        t0.elapsed().as_secs_f64()
    );
    println!(
        "Chance the map closes short of rooms: {mean:.3e} +- {se:.1e} (1 in {:.3e})",
        1.0 / mean
    );
    println!("Runs: {}", ps.iter().map(|p| format!("{p:.2e}")).collect::<Vec<_>>().join(" "));
    let levels = results.iter().map(|r| r.1.len()).max().unwrap_or(0);
    for k in 0..levels {
        let s: f64 = results.iter().map(|r| r.1.get(k).copied().unwrap_or(0.0)).sum::<f64>() / runs as f64;
        println!("  level {k}: {s:.4}");
    }
}
