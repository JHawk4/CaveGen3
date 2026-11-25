//! How close each sublevel comes to running out of treasure spots:
//! `cargo run --release -p cavegen3 --example treasure_slack -- [seeds per sublevel] [story|challenge|all]`.
//! A treasure goes in an empty room treasure spot (group 2) or an empty item alcove, and the game
//! stops placing treasures at the first one with no spot left. For random GC seeds, prints the
//! smallest number of spare spots seen, how often each small number came up, and how many
//! layouts were short of a treasure.

use cavegen3::{GameData, Layout, layout::SpawnObject};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: u32 = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(100_000);
    let which = args.get(2).map(String::as_str).unwrap_or("story");
    let data = GameData::bundled().unwrap();
    let mut names = Vec::new();
    for cave in data.caves() {
        if cave.game != "pikmin2" || cave.is_colossal_caverns() {
            continue;
        }
        let ok = match which {
            "story" => !cave.is_challenge_mode,
            "challenge" => cave.is_challenge_mode,
            _ => true,
        };
        if !ok {
            continue;
        }
        for floor in 1.. {
            let name = format!("{}-{floor}", cave.short_name());
            match data.sublevel(&name).and_then(|s| data.caveinfo(&s).map(|_| ())) {
                Ok(()) => names.push(name),
                Err(_) => break,
            }
        }
    }

    println!("sublevel  treasures  min spare  layouts with spare 0/1/2/3 (of {n})  short of a treasure");
    let mut rows: Vec<(String, u32, i64, [u64; 4], u64)> = names
        .par_iter()
        .filter_map(|name| {
            let ci = data.caveinfo(&data.sublevel(name).ok()?).ok()?;
            let want = ci.max_treasures as i64;
            if want == 0 {
                return None;
            }
            let mut min = i64::MAX;
            let mut hist = [0u64; 4];
            let mut short = 0u64;
            // Spread the seeds over the whole 32-bit space.
            for k in 0..n as u64 {
                let seed = (k.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) as u32;
                let layout = Layout::generate(seed, ci);
                let mut placed = 0i64;
                let mut free = 0i64;
                for sp in &layout.spawnpoints {
                    let unit = &layout.units[sp.unit as usize];
                    let room = unit.unit.room_type == cavegen3::caveinfo::RoomType::Room;
                    let alcove = !room && unit.unit.unit_folder_name.contains("item") && sp.group() == 9;
                    if !((room && sp.group() == 2) || alcove) {
                        continue;
                    }
                    let items = sp.contains.iter().filter(|so| matches!(so, SpawnObject::Item(_))).count() as i64;
                    placed += items;
                    // Alcove teki come after treasures, so only the exit or geyser blocks an alcove.
                    let blocked = sp.contains.iter().any(|so| match so {
                        SpawnObject::Item(_) => false,
                        SpawnObject::CapTeki(..) => !alcove,
                        _ => true,
                    });
                    if items == 0 && !blocked {
                        free += 1;
                    }
                }
                let spare = placed + free - want;
                min = min.min(spare);
                if (0..4).contains(&spare) {
                    hist[spare as usize] += 1;
                }
                if placed < want {
                    short += 1;
                }
            }
            Some((name.clone(), ci.max_treasures, min, hist, short))
        })
        .collect();
    rows.sort_by_key(|r| (r.2, std::cmp::Reverse(r.3[0]), std::cmp::Reverse(r.3[1]), std::cmp::Reverse(r.3[2])));
    for (name, want, min, h, short) in rows {
        println!("{name:<9} {want:>9}  {min:>9}  {:>7} {:>7} {:>7} {:>7}  {short}", h[0], h[1], h[2], h[3]);
    }
}
