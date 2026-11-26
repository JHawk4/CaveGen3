//! Prints where the hole is likely to be for a sublevel, seeing only the pod's room of a reference
//! seed: `cargo run --release -p cavegen3 --example hole_odds CH1-1 [ref seed hex] [layouts]`.

use cavegen3::{
    GameData, Generator, Layout,
    hole_odds::{Observations, tally},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("CH1-1");
    let reference_seed = args.get(2).map(|s| u32::from_str_radix(s, 16).unwrap()).unwrap_or(0x1234_5678);
    let n: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(20_000);
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let reference = Layout::generate(reference_seed, ci);
    let t0 = std::time::Instant::now();
    let t = tally(
        &reference,
        &Observations::default(),
        ci,
        Generator::Gc,
        (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9) as u64),
        5,
    );
    let secs = t0.elapsed().as_secs_f64();
    println!(
        "{sub} ref {reference_seed:08X}: {} tested, {} match the pod room, {:.1} us/layout",
        t.tested,
        t.matched,
        secs * 1e6 / n as f64
    );
    let total = t.consistent as f64;
    for (k, v) in &t.hole.spots {
        println!("  hole at spot {k}: {:.1}%", v / total * 100.0);
    }
    for (k, v) in &t.hole.exits {
        println!("  hole beyond door {k}: {:.1}%", v / total * 100.0);
    }
    println!("  other {:.1}, none {}", t.hole.other, t.hole.none);
    for (k, v) in &t.geyser.spots {
        println!("  geyser at spot {k}: {:.1}%", v / total * 100.0);
    }
    for (k, v) in &t.geyser.exits {
        println!("  geyser beyond door {k}: {:.1}%", v / total * 100.0);
    }
}
