//! For every sublevel with treasures, the chance its map closes before all its rooms are placed
//! (like Shower Room 7's missing boss), and how often such a map is also short of a treasure:
//! `closure_treasures [particles] [runs] [story|challenge|all] [sublevel...]`. Uses
//! `room_failure_layouts`' splitting estimate with an ideal random number generator.

use std::sync::Mutex;

use cavegen3::{GameData, Generator, SpawnObject};
use missing_treasure_analysis::splitting::room_failure_layouts;
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let particles: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(20_000);
    let runs: u64 = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(8);
    let which = args.get(3).map(String::as_str).unwrap_or("story");
    let data = GameData::bundled().unwrap();
    let mut names: Vec<String> = args.iter().skip(4).cloned().collect();
    if names.is_empty() {
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
    }
    let names: Vec<String> = names
        .into_iter()
        .filter(|n| data.caveinfo(&data.sublevel(n).unwrap()).unwrap().max_treasures > 0)
        .collect();

    // name -> (odds of each run, closed maps, closed maps short of a treasure, missing treasures, rooms seen)
    type Row = (Vec<f64>, u64, u64, u64, Vec<u32>);
    let rows: Mutex<Vec<(String, Row)>> = Mutex::new(names.iter().map(|n| (n.clone(), (vec![], 0, 0, 0, vec![]))).collect());
    let jobs: Vec<(usize, u64)> = (0..names.len()).flat_map(|i| (0..runs).map(move |r| (i, r))).collect();
    let t0 = std::time::Instant::now();
    jobs.par_iter().for_each(|&(i, r)| {
        let ci = data.caveinfo(&data.sublevel(&names[i]).unwrap()).unwrap();
        let (odds, shares, layouts) = room_failure_layouts(ci, Generator::default(), particles, 0xC105_0000 + r);
        let mut short = 0;
        let mut missing = 0;
        let mut rooms = Vec::new();
        for l in &layouts {
            let items: u32 = l
                .spawnpoints
                .iter()
                .map(|sp| sp.contains.iter().filter(|so| matches!(so, SpawnObject::Item(_))).count() as u32)
                .sum();
            if items < ci.max_treasures {
                short += 1;
                missing += (ci.max_treasures - items) as u64;
            }
            rooms.push(l.units.iter().filter(|u| u.unit.room_type == cavegen3::caveinfo::RoomType::Room).count() as u32);
        }
        let mut rows = rows.lock().unwrap();
        let row = &mut rows[i].1;
        row.0.push(odds);
        row.1 += layouts.len() as u64;
        row.2 += short;
        row.3 += missing;
        row.4.extend(rooms);
        eprintln!("{} run {r}: {odds:.2e} ({} closed, {short} short) {:.0} s; levels {}", names[i], layouts.len(), t0.elapsed().as_secs_f64(), shares.iter().map(|s| format!("{s:.3}")).collect::<Vec<_>>().join(" "));
    });

    let mut rows = rows.into_inner().unwrap();
    rows.sort_by(|a, b| {
        let m = |r: &Row| r.0.iter().sum::<f64>() / r.0.len().max(1) as f64;
        m(&b.1).partial_cmp(&m(&a.1)).unwrap()
    });
    println!("{runs} runs of {particles} maps per sublevel; chance per seed (ideal RNG) the map closes short of rooms");
    println!("sublevel  corridor%  rooms  treasures  closes short      +-        closed maps sampled  short a treasure  rooms when closed");
    for (name, (odds, closed, short, missing, rooms)) in rows {
        let ci = data.caveinfo(&data.sublevel(&name).unwrap()).unwrap();
        let n = odds.len() as f64;
        let mean = odds.iter().sum::<f64>() / n;
        let sd = (odds.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
        let (lo, hi) = (rooms.iter().min().copied().unwrap_or(0), rooms.iter().max().copied().unwrap_or(0));
        println!(
            "{name:<9} {:>9.0}  {:>5}  {:>9}  {mean:>11.2e}  {:>8.1e}  {closed:>19}  {short:>8} ({:>3.0}%) missing {missing:<4} {lo}-{hi}",
            100.0 * ci.corridor_probability,
            ci.num_rooms,
            ci.max_treasures,
            sd / n.sqrt(),
            if closed > 0 { 100.0 * short as f64 / closed as f64 } else { 0.0 },
        );
    }
}
