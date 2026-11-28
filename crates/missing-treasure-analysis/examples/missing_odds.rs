//! Chance per seed (ideal random number generator) that a sublevel's map closes with only its
//! start room and is short of a treasure: `missing_odds [gc|switch] [particles] [runs] SUBLEVEL...`.
use cavegen3::{GameData, Generator};
use missing_treasure_analysis::splitting::missing_treasure_odds;
use rayon::prelude::*;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let generator: Generator = a[1].parse().unwrap();
    let particles: usize = a[2].parse().unwrap();
    let runs: u64 = a[3].parse().unwrap();
    let data = GameData::bundled().unwrap();
    for name in &a[4..] {
        let ci = data.caveinfo(&data.sublevel(name).unwrap()).unwrap();
        let t0 = std::time::Instant::now();
        let r: Vec<_> = (0..runs).into_par_iter().map(|k| missing_treasure_odds(ci, generator, particles, 0x3155_0000 + k)).collect();
        let n = runs as f64;
        let ps: Vec<f64> = r.iter().map(|x| x.0).collect();
        let mean = ps.iter().sum::<f64>() / n;
        let se = (ps.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt() / n.sqrt();
        let close = r.iter().map(|x| x.1).sum::<f64>() / n;
        let (closed, short): (usize, usize) = r.iter().fold((0, 0), |a, x| (a.0 + x.3, a.1 + x.4));
        let hit = r.iter().filter(|x| x.3 > 0).count();
        println!("{name}: missing a treasure {mean:.2e} +- {se:.1e} (1 in {:.2e}); closes with only the start room {close:.2e}; {short} of {closed} closed maps short; {hit} of {runs} runs reached closing; {:.0} s",
            1.0 / mean, t0.elapsed().as_secs_f64());
        let levels = r.iter().map(|x| x.2.len()).max().unwrap_or(0);
        let avg: Vec<String> = (0..levels).map(|k| format!("{:.3}", r.iter().map(|x| x.2.get(k).copied().unwrap_or(0.0)).sum::<f64>() / n)).collect();
        println!("  levels: {}", avg.join(" "));
    }
}
