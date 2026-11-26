//! How often Subterranean Complex 7 has no tower room ("towerless"), and so no Gatling Groink
//! carrying the treasure.
//!
//! ```text
//! towerless scan  [start] [end]          every GC seed in start..end (hex; default all 2^31)
//! towerless random <rng> <n> [first]     n seeds from a hashed counter
//! towerless check <rng> <n>              full layouts: does "minihoudai < 2" equal "no tower"?
//! ```
//!
//! `<rng>` is `gc` or `switch` (the real generators), or `ideal-gc` / `ideal-switch`: the same
//! arithmetic fed by an ideal random stream (splitmix64, cut to 15 or 31 bits), to tell apart
//! effects of `rand()`'s resolution from effects of the LCG's own patterns. `gc-count` /
//! `switch-count` use seeds 0, 1, 2, ... and `gc-step` / `switch-step` the game RNG's states
//! stepped from 0: the web page's "c" and "g" seed modes.
//!
//! Runs on all cores; set the priority from outside (e.g. `start /belownormal`).

use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use cavegen3::{GameData, Generator, Layout, generate::placed_rooms, rng::Rng};
use rayon::prelude::*;

const SUB: &str = "SCx-7";

fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[derive(Clone, Copy, PartialEq)]
enum Source {
    Real(Generator),
    Ideal(Generator),
    /// Seeds 0, 1, 2, ... (the web page's "c").
    Count(Generator),
    /// The game RNG's states stepped from 0 (the web page's "g").
    Step(Generator),
}

impl Source {
    fn parse(s: &str) -> Source {
        match s {
            "ideal-gc" => Source::Ideal(Generator::Gc),
            "ideal-switch" => Source::Ideal(Generator::Switch),
            "gc-count" => Source::Count(Generator::Gc),
            "switch-count" => Source::Count(Generator::Switch),
            "gc-step" => Source::Step(Generator::Gc),
            "switch-step" => Source::Step(Generator::Switch),
            other => Source::Real(other.parse().unwrap()),
        }
    }
    /// The RNG for counter value `i`.
    fn rng(self, i: u64) -> (u64, Rng) {
        let seed = mix(i);
        match self {
            Source::Real(g @ Generator::Gc) => (seed & 0x7FFF_FFFF, Rng::with(g, seed & 0x7FFF_FFFF)),
            Source::Real(g) => (seed, Rng::with(g, seed)),
            Source::Ideal(g) => (seed, Rng::tape(g, Vec::new(), seed)),
            Source::Count(g) => (i, Rng::with(g, i)),
            Source::Step(g) => {
                let mut r = Rng::with(g, 0);
                r.advance(i);
                (r.state(), Rng::with(g, r.state()))
            }
        }
    }
}

/// Per-thread tallies: seeds, towerless seeds, and how many rooms the map ended up with.
#[derive(Default, Clone)]
struct Tally {
    n: u64,
    towerless: u64,
    rooms: [u64; 8],
    /// Towerless seeds whose map still has all its rooms.
    towerless_full: u64,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.n += o.n;
        self.towerless += o.towerless;
        self.towerless_full += o.towerless_full;
        for (a, b) in self.rooms.iter_mut().zip(o.rooms) {
            *a += b;
        }
    }
    fn report(&self, label: &str) {
        let p = self.towerless as f64 / self.n as f64;
        // Wilson score interval, 95%
        let (n, z) = (self.n as f64, 1.959964);
        let c = (p + z * z / (2.0 * n)) / (1.0 + z * z / n);
        let h = z / (1.0 + z * z / n) * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
        println!(
            "{label}: {} of {} towerless = {:.4}% (95% CI {:.4}%–{:.4}%); towerless with all rooms: {}",
            self.towerless,
            self.n,
            100.0 * p,
            100.0 * (c - h),
            100.0 * (c + h),
            self.towerless_full
        );
        let rooms: Vec<String> = self
            .rooms
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| format!("{k}: {n}"))
            .collect();
        println!("  rooms placed: {}", rooms.join(", "));
    }
}

fn is_tower(name: &str) -> bool {
    name.contains("tower")
}

fn classify(rng: Rng, ci: &cavegen3::caveinfo::CaveInfo, t: &mut Tally) -> bool {
    let rooms = placed_rooms(rng, ci);
    t.n += 1;
    t.rooms[rooms.len().min(7)] += 1;
    let towerless = !rooms.iter().any(|u| is_tower(&u.unit_folder_name));
    if towerless {
        t.towerless += 1;
        if rooms.len() as u32 == ci.num_rooms {
            t.towerless_full += 1;
        }
    }
    towerless
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(SUB).unwrap()).unwrap();
    let t0 = std::time::Instant::now();
    match args.get(1).map(String::as_str).unwrap_or("scan") {
        "scan" => {
            let hex = |i: usize, d: u64| {
                args.get(i)
                    .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
                    .unwrap_or(d)
            };
            let (start, end) = (hex(2, 0), hex(3, 1 << 31));
            let total = Mutex::new(Tally::default());
            let examples = Mutex::new(Vec::new());
            let done = AtomicU64::new(0);
            let chunk = 1u64 << 20;
            let chunks: Vec<u64> = (start..end).step_by(chunk as usize).collect();
            chunks.par_iter().for_each(|&c0| {
                let mut t = Tally::default();
                let mut ex = Vec::new();
                for seed in c0..(c0 + chunk).min(end) {
                    if classify(Rng::with(Generator::Gc, seed), ci, &mut t) && ex.len() < 2 {
                        ex.push(seed);
                    }
                }
                total.lock().unwrap().add(&t);
                examples.lock().unwrap().extend(ex);
                let n = done.fetch_add(t.n, Ordering::Relaxed) + t.n;
                if (n / chunk) % 64 == 0 {
                    let s = t0.elapsed().as_secs_f64();
                    let tt = total.lock().unwrap();
                    eprintln!(
                        "{n} of {} seeds, {s:.0} s, {:.0}/s, towerless so far {:.4}%",
                        end - start,
                        n as f64 / s,
                        100.0 * tt.towerless as f64 / tt.n as f64
                    );
                }
            });
            let mut ex = examples.into_inner().unwrap();
            ex.sort();
            ex.truncate(10);
            total.lock().unwrap().report(&format!("GC seeds {start:#X}..{end:#X}"));
            println!(
                "  some towerless seeds: {}",
                ex.iter().map(|s| format!("{s:08X}")).collect::<Vec<_>>().join(" ")
            );
        }
        "random" => {
            let source = Source::parse(&args[2]);
            let n: u64 = args[3].parse().unwrap();
            let first: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(0);
            let chunk = 1u64 << 16;
            let total = Mutex::new(Tally::default());
            let examples = Mutex::new(Vec::new());
            (0..n.div_ceil(chunk)).into_par_iter().for_each(|c| {
                let mut t = Tally::default();
                let mut ex = Vec::new();
                for i in first + c * chunk..first + ((c + 1) * chunk).min(n) {
                    let (seed, rng) = source.rng(i);
                    if classify(rng, ci, &mut t) && ex.len() < 1 {
                        ex.push(seed);
                    }
                }
                total.lock().unwrap().add(&t);
                examples.lock().unwrap().extend(ex);
            });
            let mut ex = examples.into_inner().unwrap();
            ex.truncate(8);
            total
                .lock()
                .unwrap()
                .report(&format!("{} counters {first}..{}", args[2], first + n));
            println!(
                "  some towerless seeds: {}",
                ex.iter().map(|s| format!("{s:X}")).collect::<Vec<_>>().join(" ")
            );
        }
        "check" => {
            let source = Source::parse(&args[2]);
            let Source::Real(g) = source else {
                panic!("check needs gc or switch")
            };
            let n: u64 = args[3].parse().unwrap();
            // [tower present?][minihoudai < 2?]
            let table = Mutex::new([[0u64; 2]; 2]);
            let odd = Mutex::new(Vec::new());
            (0..n.div_ceil(1 << 14)).into_par_iter().for_each(|c| {
                let mut tab = [[0u64; 2]; 2];
                for i in c << 14..((c + 1) << 14).min(n) {
                    let (seed, rng) = source.rng(i);
                    let towerless = classify(rng, ci, &mut Tally::default());
                    let layout = Layout::generate_with(g, seed, ci);
                    let has_tower = layout.units.iter().any(|u| is_tower(&u.unit.unit_folder_name));
                    assert_eq!(has_tower, !towerless, "seed {seed:X}: room-only check disagrees with the layout");
                    let few = layout.count("minihoudai") < 2; // Caveripper's "scx7 minihoudai < 2"
                    assert_eq!(layout.count("sinkukan_b") == 0, !has_tower, "seed {seed:X}: Indomitable CPU");
                    tab[has_tower as usize][few as usize] += 1;
                    if has_tower == few {
                        odd.lock().unwrap().push(seed);
                    }
                }
                let mut t = table.lock().unwrap();
                for a in 0..2 {
                    for b in 0..2 {
                        t[a][b] += tab[a][b];
                    }
                }
            });
            let t = table.into_inner().unwrap();
            println!("{} seeds of {SUB} ({}), full layouts:", n, args[2]);
            println!("  no tower,  Gatling Groinks < 2: {}", t[0][1]);
            println!("  no tower,  Gatling Groinks >= 2: {}", t[0][0]);
            println!("  tower,     Gatling Groinks < 2: {}", t[1][1]);
            println!("  tower,     Gatling Groinks >= 2: {}", t[1][0]);
            let odd = odd.into_inner().unwrap();
            println!("  seeds where they disagree (first 10): {:X?}", &odd[..odd.len().min(10)]);
        }
        other => panic!("unknown mode {other}"),
    }
    eprintln!("{:.1} s", t0.elapsed().as_secs_f64());
}
