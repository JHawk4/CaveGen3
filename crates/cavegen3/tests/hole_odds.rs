//! Hole odds (`cavegen3::hole_odds`): the chance of the exit at each spot, from the finished
//! layout, checked against where the generator really put it.

use cavegen3::{GameData, Layout, SpawnObject, caveinfo::CaveInfo, hole_odds::exit_odds};

fn seeds(n: u64) -> impl Iterator<Item = u32> {
    (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9).wrapping_add(0x1234))
}

fn is_exit(so: &SpawnObject) -> bool {
    matches!(so, SpawnObject::Hole(_) | SpawnObject::Geyser(_))
}

fn exit_of(l: &Layout) -> u32 {
    l.hole.or(l.geyser).unwrap()
}

fn challenge_floors(data: &GameData) -> Vec<&CaveInfo> {
    data.caves()
        .iter()
        .filter(|c| c.is_challenge_mode && c.game == "pikmin2")
        .flat_map(|c| data.floors(c).unwrap().iter())
        .collect()
}

/// On every Challenge Mode floor the odds add up to 1, sit only on empty alcoves and dead ends
/// clear of hard (200) and special (150) teki, and never rule out the real exit: no evidence
/// (treasures, cap teki, gates) is ever wrongly taken as certain.
#[test]
fn never_rules_out_the_real_exit() {
    let data = GameData::bundled().unwrap();
    for ci in challenge_floors(&data) {
        let name = ci.name();
        for seed in seeds(150) {
            let l = Layout::generate(seed, ci);
            let odds = exit_odds(&l);
            let sum: f64 = odds.values().sum();
            assert!((sum - 1.0).abs() < 1e-9, "{name} {seed:#X}: {sum}");
            let p = odds.get(&exit_of(&l)).copied().unwrap_or(0.0);
            assert!(p > 0.0, "{name} {seed:#X}: the real exit was ruled out");
            let teki: Vec<_> = l
                .spawnpoints
                .iter()
                .filter(|sp| matches!(sp.group(), 1 | 8) && sp.contains.iter().any(|so| matches!(so, SpawnObject::Teki(..))))
                .collect();
            for &i in odds.keys() {
                let sp = &l.spawnpoints[i as usize];
                assert!(matches!(sp.group(), 4 | 9) && sp.contains.iter().all(is_exit), "{name} {seed:#X}");
                for t in &teki {
                    let gap = if t.group() == 1 { 200.0 } else { 150.0 };
                    assert!(t.pos.p2_dist(&sp.pos) >= gap, "{name} {seed:#X}");
                }
            }
        }
    }
}

/// The odds are calibrated and sharper than the hole weights alone. Calibrated: over many seeds,
/// the chance given to the real exit averages to the sum of squared chances (what it should be if
/// the odds are right), within a few percent (cap teki are only modelled as a random choice).
/// Sharper: the real exit gets a higher chance on average than from the weights alone.
#[test]
fn odds_are_calibrated_and_sharper() {
    let data = GameData::bundled().unwrap();
    let (mut hit, mut expect, mut prior_hit, mut n) = (0.0, 0.0, 0.0, 0.0);
    for ci in challenge_floors(&data) {
        for seed in seeds(60) {
            let l = Layout::generate(seed, ci);
            let odds = exit_odds(&l);
            let exit = exit_of(&l);
            hit += odds[&exit];
            expect += odds.values().map(|p| p * p).sum::<f64>();
            // The weights alone, over the empty candidates.
            let cands: Vec<(usize, f64)> = l
                .spawnpoints
                .iter()
                .enumerate()
                .filter(|(_, sp)| matches!(sp.group(), 4 | 9) && sp.hole_score > 0 && sp.contains.iter().all(is_exit))
                .map(|(i, sp)| (i, sp.hole_score as f64))
                .collect();
            let total: f64 = cands.iter().map(|c| c.1).sum();
            prior_hit += cands.iter().find(|c| c.0 == exit as usize).unwrap().1 / total;
            n += 1.0;
        }
    }
    let (hit, expect, prior_hit) = (hit / n, expect / n, prior_hit / n);
    println!("real exit's chance: {hit:.3} (predicted {expect:.3}); from the weights alone {prior_hit:.3}");
    assert!((hit - expect).abs() < 0.03, "{hit:.3} vs {expect:.3}");
    assert!(hit > prior_hit + 0.02, "{hit:.3} vs {prior_hit:.3}");
}

/// Story mode has no odds: the best score always wins.
#[test]
fn none_in_story_mode() {
    let data = GameData::bundled().unwrap();
    let ci = data.caveinfo(&data.sublevel("SCx-6").unwrap()).unwrap();
    assert!(exit_odds(&Layout::generate(0x1234, ci)).is_empty());
}
