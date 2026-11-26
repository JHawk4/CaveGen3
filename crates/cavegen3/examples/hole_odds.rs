//! Prints where the hole is likely to be for a sublevel, seeing only the pod's room of a reference
//! seed, two ways: from the reference's own hole weights (no other layouts), and by counting
//! random layouts with the same pod room.
//! `cargo run --release -p cavegen3 --example hole_odds CH12-1 [ref seed hex] [layouts]`.

use cavegen3::{
    GameData, Generator, Layout,
    generate::generate_partly,
    hole_odds::{Observations, Where, tally, weighted},
    rng::Rng,
};

fn show(label: &str, w: &Where, n: f64) {
    println!("{label}");
    for (k, v) in &w.spots {
        println!("  at spot {k}: {:.1}%", v / n * 100.0);
    }
    for (k, v) in &w.exits {
        println!("  beyond door {k}: {:.1}%", v / n * 100.0);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("CH12-1");
    let reference_seed = args.get(2).map(|s| u32::from_str_radix(s, 16).unwrap()).unwrap_or(0x1234_5678);
    let n: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(20_000);
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let obs = Observations::default();

    let t0 = std::time::Instant::now();
    let at_exits = generate_partly(Rng::new(reference_seed), ci, None, true).unwrap();
    let w = weighted(&at_exits, &obs);
    println!(
        "{sub} ref {reference_seed:08X}, from its weights ({:.0} us):",
        t0.elapsed().as_secs_f64() * 1e6
    );
    show(" hole", &w.hole, 1.0);
    show(" geyser", &w.geyser, 1.0);

    let reference = Layout::generate(reference_seed, ci);
    let t0 = std::time::Instant::now();
    let t = tally(
        &reference,
        &obs,
        ci,
        Generator::Gc,
        (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9) as u64),
        5,
    );
    println!(
        "counting {} layouts ({} with this pod room, {:.1} s):",
        t.tested,
        t.matched,
        t0.elapsed().as_secs_f64()
    );
    let m = t.consistent as f64;
    show(" hole", &t.hole, m);
    show(" geyser", &t.geyser, m);
}
