//! Prints each map unit placement step for one seed:
//! `cargo run --release -p cavegen3 --example room_trace -- SR-7 31D70855 [gc|switch]`.

use cavegen3::{GameData, Generator, generate::trace_rooms_placed, rng::Rng};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = &args[1];
    let seed = u64::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap();
    let generator: Generator = args.get(3).map(|s| s.parse().unwrap()).unwrap_or_default();
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel(sub).unwrap()).unwrap();
    let (rooms, steps) = trace_rooms_placed(Rng::with(generator, seed), ci);
    println!("{sub} seed {seed:#X}: {rooms} of {} rooms after {steps} steps", ci.num_rooms);
}
