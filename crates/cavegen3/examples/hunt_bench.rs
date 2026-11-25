//! Single-thread timing of the ways to test a seed when hunting maps that close with only the
//! start room, and a check that the fast filter never rules out a seed it shouldn't:
//! `hunt_bench SUBLEVEL [gc|switch] [seeds]`.

use cavegen3::{
    GameData, Generator,
    generate::{ClosureFilter, closes_with_rooms, profile_closes_with_rooms, rooms_placed},
    rng::Rng,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(String::as_str).unwrap_or("WFG-4");
    let generator: Generator = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(Generator::Switch);
    let n: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(200_000);
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    println!("{sub}: {} rooms, corridor probability {}", ci.num_rooms, ci.corridor_probability);
    let seed = |i: u64| i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1234_5678_9ABC_DEF0;
    let per = |t: std::time::Instant| t.elapsed().as_secs_f64() * 1e6 / n as f64;

    let (t, steps) = profile_closes_with_rooms(ci, generator, n, 1);
    let us = |x: f64| x * 1e6 / n as f64;
    println!(
        "stages (us/seed): new {:.2}, sort+teki {:.2}, start room {:.2}, steps {:.2}; {steps:.1} steps/seed",
        us(t[0]),
        us(t[1]),
        us(t[2]),
        us(t[3])
    );

    let filter = ClosureFilter::new(ci, generator);
    println!("filter: {filter:?}");

    // Every seed the filter rules out must also fail the full test.
    if let Some(f) = filter {
        let mut rejected = 0u64;
        for i in 0..n {
            if f.rejects(seed(i)) {
                rejected += 1;
                let rooms = cavegen3::generate::first_step_rooms(Rng::with(generator, seed(i)), ci);
                assert!(rooms >= 2, "filter rejected seed {:016X}, but its first step placed no room", seed(i));
            }
        }
        println!("filter rules out {:.1}% of seeds; all checked against the first placement step", 100.0 * rejected as f64 / n as f64);
    }

    let t = std::time::Instant::now();
    let mut x = 0u64;
    for i in 0..n {
        x += rooms_placed(Rng::with(generator, seed(i)), ci).0 as u64;
    }
    let full = per(t);
    let t = std::time::Instant::now();
    let mut y = 0u64;
    for i in 0..n {
        y += closes_with_rooms(Rng::with(generator, seed(i)), ci, 1) as u64;
    }
    let capped = per(t);
    println!("current hunt (place all rooms): {full:.2} us/seed ({:.0}/s)", 1e6 / full);
    println!("stop at the 2nd room:           {capped:.2} us/seed ({:.0}/s)", 1e6 / capped);
    if let Some(f) = filter {
        let t = std::time::Instant::now();
        for i in 0..n {
            let s = seed(i);
            if !f.rejects(s) {
                y += closes_with_rooms(Rng::with(generator, s), ci, 1) as u64;
            }
        }
        let filtered = per(t);
        println!("filter, then stop at 2nd room:  {filtered:.2} us/seed ({:.0}/s)", 1e6 / filtered);
    }
    let _ = (x, y);
}
