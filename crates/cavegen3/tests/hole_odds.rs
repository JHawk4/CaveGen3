//! Challenge Mode hole odds (`cavegen3::hole_odds`): the candidates and weights are the
//! generator's, marks rule spots out and renormalize the rest, and the odds are calibrated against
//! where the exit really goes.

use cavegen3::{
    GameData, Layout, SpawnObject,
    caveinfo::CaveInfo,
    generate::generate_partly,
    hole_odds::{Fact, exit_candidates, exit_of, odds},
    rng::Rng,
};

fn seeds(n: u64) -> impl Iterator<Item = u32> {
    (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9).wrapping_add(0x1234))
}

fn caveinfo<'a>(data: &'a GameData, name: &str) -> &'a CaveInfo {
    data.caveinfo(&data.sublevel(name).unwrap()).unwrap()
}

/// The layout as it was when the exit was picked (unit scores before teki).
fn at_exits(seed: u32, ci: &CaveInfo) -> Layout<'_> {
    generate_partly(Rng::new(seed), ci, true)
}

/// The candidates and weights are the ones the generator used: in Challenge Mode the spots that
/// got a hole score, with it as the weight; the exit always lands on one.
#[test]
fn weights_are_the_generators() {
    let data = GameData::bundled().unwrap();
    for name in ["CH1-1", "CH12-1", "CH20-1", "CH5-2"] {
        let ci = caveinfo(&data, name);
        for seed in 0..300u32 {
            let l = at_exits(seed, ci);
            let mut cands = exit_candidates(&l);
            cands.sort_by_key(|c| c.0);
            let scored: Vec<(u32, f64)> = l
                .spawnpoints
                .iter()
                .enumerate()
                .filter(|(_, sp)| sp.hole_score > 0)
                .map(|(i, sp)| (i as u32, sp.hole_score as f64))
                .collect();
            assert_eq!(cands, scored, "{name} {seed:#X}");
            let exit = exit_of(&l).unwrap();
            assert!(cands.iter().any(|c| c.0 == exit && c.1 > 0.0), "{name} {seed:#X}");
        }
    }
    // Story mode: the hole is on one of the best-scored spots.
    let ci = caveinfo(&data, "SCx-6");
    for seed in 0..300u32 {
        let l = at_exits(seed, ci);
        let hole = l.hole.unwrap();
        assert!(exit_candidates(&l).iter().any(|c| c.0 == hole && c.1 == 1.0), "{seed:#X}");
    }
}

/// Each spot gets weight / total; ruling one out renormalizes the rest exactly; saying the exit
/// is somewhere gives it 1; ruling everything out leaves nothing.
#[test]
fn marks_renormalize() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    let l = at_exits(0x1234_5678, ci);
    let cands = exit_candidates(&l);
    let total: f64 = cands.iter().map(|c| c.1).sum();
    let base = odds(&l, &[]);
    assert_eq!(base.len(), cands.len());
    for &(i, w) in &cands {
        assert!((base[&i] - w / total).abs() < 1e-12);
    }
    let (out, w_out) = cands[0];
    for fact in [Fact::NoExit, Fact::Empty, Fact::NothingVisible, Fact::Occupied] {
        let ruled = odds(&l, &[(out, fact)]);
        assert!(!ruled.contains_key(&out));
        for &(i, w) in &cands[1..] {
            assert!((ruled[&i] - w / (total - w_out)).abs() < 1e-12);
        }
    }
    let here = odds(&l, &[(out, Fact::Exit)]);
    assert_eq!(here.len(), 1);
    assert!((here[&out] - 1.0).abs() < 1e-12);
    let all: Vec<(u32, Fact)> = cands.iter().map(|c| (c.0, Fact::Empty)).collect();
    assert!(odds(&l, &all).is_empty());
}

/// The odds are calibrated against real generation: over many seeds, the chance each layout gave
/// its actual exit spot averages to the sum of its squared chances (what it should be if the odds
/// are right). Hole floors and a final floor (geyser).
#[test]
fn odds_are_calibrated() {
    let data = GameData::bundled().unwrap();
    for name in ["CH12-1", "CH20-1", "CH1-1", "CH5-2"] {
        let ci = caveinfo(&data, name);
        let (mut hit, mut expect, mut var) = (0.0, 0.0, 0.0);
        for seed in seeds(3000) {
            let l = at_exits(seed, ci);
            let p = odds(&l, &[]);
            let sq: f64 = p.values().map(|p| p * p).sum();
            let here = p.get(&exit_of(&l).unwrap()).copied().unwrap_or(0.0);
            assert!(here > 0.0, "{name} {seed:#X}: the real spot had no chance");
            hit += here;
            expect += sq;
            var += p.values().map(|p| p * p * p).sum::<f64>() - sq * sq;
        }
        let sd = var.max(0.0).sqrt();
        assert!((hit - expect).abs() < 5.0 * sd + 1e-6, "{name}: {hit:.2} vs {expect:.2} (sd {sd:.2})");
    }
}

/// Marks taken from the full layout never rule out the real exit: spots seen empty or holding
/// something else aren't it, nothing visible in the comedown means no exit there, and a teki in a
/// hard or special teki spot keeps the exit 200 or 150 away.
#[test]
fn true_marks_keep_the_real_exit() {
    let data = GameData::bundled().unwrap();
    for name in ["CH12-1", "CH20-1", "CH5-2"] {
        let ci = caveinfo(&data, name);
        let mut ruled_out_something = false;
        for seed in seeds(500) {
            let full = Layout::generate(seed, ci);
            let l = at_exits(seed, ci);
            let exit = exit_of(&l).unwrap();
            let facts: Vec<(u32, Fact)> = full
                .spawnpoints
                .iter()
                .enumerate()
                .filter_map(|(i, sp)| {
                    let fact = [Fact::Occupied, Fact::Empty, Fact::NothingVisible]
                        .into_iter()
                        .find(|f| f.holds(&sp.contains))?;
                    Some((i as u32, fact))
                })
                .collect();
            let p = odds(&l, &facts);
            assert!(p.get(&exit).is_some_and(|&p| p > 0.0), "{name} {seed:#X}");
            ruled_out_something |= p.len() < exit_candidates(&l).len();
            for &(i, _) in facts.iter().filter(|f| f.1 == Fact::Occupied) {
                let sp = &full.spawnpoints[i as usize];
                if sp.group() == 1 && sp.contains.iter().any(|so| matches!(so, SpawnObject::Teki(..))) {
                    assert!(sp.pos.p2_dist(&full.spawnpoints[exit as usize].pos) >= 200.0);
                }
            }
        }
        assert!(ruled_out_something, "{name}");
    }
}

#[test]
fn facts_parse() {
    let facts = Fact::parse_list("12e,40h,7v,3o,9n").unwrap();
    assert_eq!(
        facts,
        vec![(12, Fact::Empty), (40, Fact::Exit), (7, Fact::NothingVisible), (3, Fact::Occupied), (9, Fact::NoExit)]
    );
    assert!(Fact::parse_list("-").unwrap().is_empty());
    assert!(Fact::parse_list("").unwrap().is_empty());
    assert!(Fact::parse_list("12q").is_err());
    assert!(Fact::parse_list("xe").is_err());
}
