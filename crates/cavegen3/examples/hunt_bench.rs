//! Single-thread timing of the ways to test a seed when hunting maps that close with only the
//! start room, and a check that the fast filter never rules out a seed it shouldn't:
//! `hunt_bench SUBLEVEL [gc|switch] [seeds]`.

use cavegen3::{
    GameData, Generator,
    generate::{ClosureFilter, closes_with_rooms, profile_closes_with_rooms, rooms_after_steps, rooms_placed},
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
    println!("{sub} {generator:?}: {} rooms, corridor probability {}", ci.num_rooms, ci.corridor_probability);
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

    let t = std::time::Instant::now();
    let filter = ClosureFilter::new(ci, generator);
    println!(
        "filter: {}, built in {:.2} s",
        match &filter {
            Some(f) if f.checks_second_step() => "first two steps",
            Some(_) => "first step only",
            None => "none",
        },
        t.elapsed().as_secs_f64()
    );

    // Every seed the filter rules out must place a second room in the steps it looked at.
    if let Some(f) = &filter {
        let (mut first, mut second) = (0u64, 0u64);
        for i in 0..n {
            let s = seed(i);
            if f.rejects_first_step(s) {
                first += 1;
                let rooms = rooms_after_steps(Rng::with(generator, s), ci, 1);
                assert!(rooms >= 2, "filter rejected seed {s:016X} at step 1, but it placed no room there");
            } else if f.rejects(s) {
                second += 1;
                let rooms = rooms_after_steps(Rng::with(generator, s), ci, 2);
                assert!(rooms >= 2, "filter rejected seed {s:016X} at step 2, but it placed no room by then");
            }
        }
        println!(
            "rules out {:.1}% at step 1 and {:.1}% more at step 2 ({:.1}% left); all checked against the generator",
            100.0 * first as f64 / n as f64,
            100.0 * second as f64 / n as f64,
            100.0 * (n - first - second) as f64 / n as f64
        );
    }

    let t = std::time::Instant::now();
    let mut x = 0u64;
    for i in 0..n {
        x += rooms_placed(Rng::with(generator, seed(i)), ci).0 as u64;
    }
    let full = per(t);
    let t = std::time::Instant::now();
    for i in 0..n {
        x += closes_with_rooms(Rng::with(generator, seed(i)), ci, 1) as u64;
    }
    let capped = per(t);
    println!("current hunt (place all rooms): {full:.2} us/seed ({:.0}/s)", 1e6 / full);
    println!("stop at the 2nd room:           {capped:.2} us/seed ({:.0}/s)", 1e6 / capped);
    if let Some(f) = &filter {
        let t = std::time::Instant::now();
        for i in 0..n {
            let s = seed(i);
            if !f.rejects_first_step(s) {
                x += closes_with_rooms(Rng::with(generator, s), ci, 1) as u64;
            }
        }
        let one = per(t);
        println!("step-1 filter, then the rest:   {one:.2} us/seed ({:.0}/s)", 1e6 / one);
        let t = std::time::Instant::now();
        for i in 0..n {
            let s = seed(i);
            if !f.rejects(s) {
                x += closes_with_rooms(Rng::with(generator, s), ci, 1) as u64;
            }
        }
        let two = per(t);
        println!("step-1+2 filter, then the rest: {two:.2} us/seed ({:.0}/s)", 1e6 / two);
    }
    std::hint::black_box(x);
}
