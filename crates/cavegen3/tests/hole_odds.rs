//! Hole odds (`cavegen3::hole_odds`): the chance of the exit at each spot, from the finished
//! layout.

use cavegen3::{GameData, Layout, SpawnObject, hole_odds::exit_odds};

fn seeds(n: u64) -> impl Iterator<Item = u32> {
    (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9).wrapping_add(0x1234))
}

fn is_exit(so: &SpawnObject) -> bool {
    matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_))
}

/// The odds add up to 1, sit only on empty alcoves and dead ends clear of hard (200) and special
/// (150) teki, are the hole scores renormalized, and always include the real exit.
#[test]
fn odds_follow_the_generator() {
    let data = GameData::bundled().unwrap();
    for name in ["CH1-1", "CH12-1", "CH20-1", "CH5-2", "CH30-1", "CH26-2"] {
        let ci = data.caveinfo(&data.sublevel(name).unwrap()).unwrap();
        for seed in seeds(300) {
            let l = Layout::generate(seed, ci);
            let odds = exit_odds(&l);
            let sum: f64 = odds.values().sum();
            assert!((sum - 1.0).abs() < 1e-9, "{name} {seed:#X}");
            for exit in [l.hole, l.geyser].into_iter().flatten() {
                assert!(odds.get(&exit).is_some_and(|&p| p > 0.0), "{name} {seed:#X}: the real exit was left out");
            }
            let teki: Vec<_> = l
                .spawnpoints
                .iter()
                .filter(|sp| matches!(sp.group(), 1 | 8) && sp.contains.iter().any(|so| matches!(so, SpawnObject::Teki(..))))
                .collect();
            let weights: f64 = odds.keys().map(|&i| l.spawnpoints[i as usize].hole_score as f64).sum();
            for (&i, &p) in &odds {
                let sp = &l.spawnpoints[i as usize];
                assert!(matches!(sp.group(), 4 | 9) && sp.contains.iter().all(is_exit), "{name} {seed:#X}");
                for t in &teki {
                    let gap = if t.group() == 1 { 200.0 } else { 150.0 };
                    assert!(t.pos.p2_dist(&sp.pos) >= gap, "{name} {seed:#X}");
                }
                assert!((p - sp.hole_score as f64 / weights).abs() < 1e-12, "{name} {seed:#X}");
            }
        }
    }
}

/// Story mode has no odds: the best score always wins.
#[test]
fn none_in_story_mode() {
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel("SCx-6").unwrap()).unwrap();
    assert!(exit_odds(&Layout::generate(0x1234, ci)).is_empty());
}
