//! How tightly a real seed's rand() values are pinned by its map closing short of rooms:
//! `seed_bits SR-7 31D70855`. For each call up to the end of treasure placement, the widest run
//! of values (others unchanged) that still closes short; prints the bits per call range.
use cavegen3::{GameData, Generator, rng::Rng};
use missing_treasure_analysis::steer;
use rayon::prelude::*;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(&a[1]).unwrap()).unwrap();
    let mut rng = Rng::new(u32::from_str_radix(&a[2], 16).unwrap());
    let tape: Vec<u32> = (0..40_000).map(|_| rng.rand_raw()).collect();
    let base = steer::run_tape(ci, Generator::Gc, &tape);
    println!("{base:?}");
    let ok = |i: usize, x: u32| { let mut t = tape.clone(); t[i] = x; let o = steer::run_tape(ci, Generator::Gc, &t); o.rooms < o.num_rooms };
    let bits: Vec<(usize, f64)> = (0..base.calls).into_par_iter().map(|i| {
        let v = tape[i];
        let good: Vec<bool> = (0..32768u32).step_by(8).map(|x| ok(i, x)).collect();
        // width of the run of good samples around v
        let k = (v / 8) as usize;
        let (mut lo, mut hi) = (k, k);
        while lo > 0 && good[lo - 1] { lo -= 1 }
        while hi + 1 < good.len() && good[hi + 1] { hi += 1 }
        let any = good.iter().filter(|g| **g).count();
        let _ = any;
        (i, (good.len() as f64 / (hi - lo + 1) as f64).log2())
    }).collect();
    let total: f64 = bits.iter().map(|b| b.1).sum();
    let n = bits.iter().filter(|b| b.1 > 0.001).count();
    println!("{} calls, {n} constrained, {total:.1} bits", base.calls);
    let mut by = std::collections::BTreeMap::new();
    for (i, b) in &bits { let e = by.entry(i / 100).or_insert((0, 0.0)); if *b > 0.001 { e.0 += 1 } e.1 += b; }
    for (k, (c, b)) in by { if b > 0.0 { println!("  calls {:>5}-{:<5} {c:>3} constrained {b:>7.1} bits", k * 100, k * 100 + 99) } }
}
