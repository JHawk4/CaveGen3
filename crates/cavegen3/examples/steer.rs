//! Chooses what `rand()` returns to make a sublevel's map close before all its rooms are placed
//! and be short of a treasure, then measures how tightly each value is constrained:
//! `steer [gc|switch] [beam] [tries] [restarts] SUBLEVEL...`.
//!
//! For each call on the found tape, the widest run of values around the chosen one that still
//! gives such a map (all other calls unchanged) is that call's constraint; log2(range / width)
//! is its cost in bits. The total is roughly how many bits of seed the map needs, which says
//! whether a 64-bit Switch seed with it can exist and be found by lattice reduction.

use cavegen3::{GameData, Generator, generate::steer, rng::Rng};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let generator: Generator = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(Generator::Switch);
    let beam: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(64);
    let tries: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(16);
    let restarts: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(20);
    let data = GameData::bundled().unwrap();

    // The tape must reproduce real generation: SR-7 31D70855 (GC) closes with one room.
    {
        let ci = data.caveinfo(&data.sublevel("SR-7").unwrap()).unwrap();
        let mut rng = Rng::new(0x31D7_0855);
        let tape: Vec<u32> = (0..20_000).map(|_| rng.rand_raw()).collect();
        let o = steer::run_tape(ci, Generator::Gc, &tape);
        assert_eq!(o.rooms, 1, "tape playback differs from the LCG: {o:?}");
    }

    let range: u64 = match generator {
        Generator::Gc => 1 << 15,
        Generator::Switch => 1 << 31,
    };
    for name in &args[5.min(args.len())..] {
        let ci = data.caveinfo(&data.sublevel(name).unwrap()).unwrap();
        let t0 = std::time::Instant::now();
        // Several independent searches; keep every tape found.
        let found: Vec<(Vec<u32>, steer::SearchStats)> = (0..16u64)
            .into_par_iter()
            .map(|k| steer::search(ci, generator, beam, tries, restarts, 0x57EE_0000 + k))
            .map(|(t, s)| (t.unwrap_or_default(), s))
            .collect();
        let (closures, missing): (u64, u64) = found.iter().fold((0, 0), |a, f| (a.0 + f.1.closures, a.1 + f.1.missing));
        let tapes: Vec<&Vec<u32>> = found.iter().map(|f| &f.0).filter(|t| !t.is_empty()).collect();
        println!(
            "{name}: {} of 16 searches found a closed map short of a treasure ({closures} closures seen), {:.0} s",
            tapes.len(),
            t0.elapsed().as_secs_f64()
        );
        let _ = missing;
        for (k, tape) in tapes.iter().enumerate().take(3) {
            let base = steer::run_tape(ci, generator, tape);
            assert!(base.short_and_missing(), "{base:?}");
            let calls = base.calls;
            let ok = |i: usize, x: u32| {
                let mut t = tape.to_vec();
                t[i] = x;
                steer::run_tape(ci, generator, &t).short_and_missing()
            };
            let bits: Vec<(usize, u32, u32, f64)> = (0..calls)
                .into_par_iter()
                .map(|i| {
                    let v = tape[i];
                    // Lower edge: the smallest lo with lo..=v all good, assuming one run.
                    let lo = if ok(i, 0) {
                        0
                    } else {
                        let (mut bad, mut good) = (0u32, v);
                        while good - bad > 1 {
                            let m = bad + (good - bad) / 2;
                            if ok(i, m) { good = m } else { bad = m }
                        }
                        good
                    };
                    let top = (range - 1) as u32;
                    let hi = if ok(i, top) {
                        top
                    } else {
                        let (mut good, mut bad) = (v, top);
                        while bad - good > 1 {
                            let m = good + (bad - good) / 2;
                            if ok(i, m) { good = m } else { bad = m }
                        }
                        good
                    };
                    let width = (hi - lo) as f64 + 1.0;
                    (i, lo, hi, (range as f64 / width).log2())
                })
                .collect();
            let total: f64 = bits.iter().map(|b| b.3).sum();
            let constrained: Vec<&(usize, u32, u32, f64)> = bits.iter().filter(|b| b.3 > 0.001).collect();
            println!(
                "  tape {k}: {} rooms of {}, {} of {} treasures; {calls} calls, {} constrained, {total:.1} bits",
                base.rooms,
                base.num_rooms,
                base.treasures,
                base.max_treasures,
                constrained.len()
            );
            let mut top = constrained.clone();
            top.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
            println!(
                "    biggest: {}",
                top.iter()
                    .take(12)
                    .map(|b| format!("#{} {:.1}", b.0, b.3))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let path = format!("steer_{name}_{k}.txt");
            let text: String = bits.iter().map(|b| format!("{} {} {} {} {:.3}\n", b.0, tape[b.0], b.1, b.2, b.3)).collect();
            std::fs::write(&path, text).unwrap();
        }
    }
}
