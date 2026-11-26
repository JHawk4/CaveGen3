//! Challenge Mode hole odds (`cavegen3::hole_odds`): the conditioning, checked against direct
//! counts over the same seeds and against the hole's weighted pick.

use cavegen3::{
    GameData, Generator, Layout, SpawnObject,
    caveinfo::CaveInfo,
    generate::generate_partly,
    hole_odds::{Fact, Observations, Tally, exit_candidates, match_units, remap, tally, weighted},
    rng::Rng,
};

const N: u64 = 4000;

fn seeds(n: u64) -> impl Iterator<Item = u64> {
    (0..n).map(|i| (i as u32).wrapping_mul(0x9E37_79B9).wrapping_add(0x1234) as u64)
}

fn caveinfo<'a>(data: &'a GameData, name: &str) -> &'a CaveInfo {
    data.caveinfo(&data.sublevel(name).unwrap()).unwrap()
}

fn run(reference: &Layout, obs: &Observations, ci: &CaveInfo) -> Tally {
    tally(reference, obs, ci, Generator::Gc, seeds(N), 3)
}

fn total(t: &Tally) -> f64 {
    t.hole.spots.values().sum::<f64>() + t.hole.exits.values().sum::<f64>() + t.hole.other + t.hole.none
}

/// With only the pod's room seen, a layout matches when the pod is in the same unit at the same
/// spot, and the hole's spot in that unit is counted directly: the same numbers as `tally`.
#[test]
fn pod_room_odds_match_a_direct_count() {
    let data = GameData::bundled().unwrap();
    for name in ["CH1-1", "CH12-1", "CH20-1"] {
        let ci = caveinfo(&data, name);
        let reference = Layout::generate(0x1234_5678, ci);
        let t = run(&reference, &Observations::default(), ci);

        let r0 = &reference.units[0];
        let ship = reference.ship.unwrap() - r0.first_spawnpoint;
        let (mut matched, mut in_room) = (0u64, std::collections::BTreeMap::<u32, f64>::new());
        for seed in seeds(N) {
            let l = Layout::generate(seed as u32, ci);
            if l.units[0].unit.name_id != r0.unit.name_id || l.ship.unwrap() - l.units[0].first_spawnpoint != ship {
                continue;
            }
            matched += 1;
            let hole = l.hole.unwrap();
            if l.spawnpoints[hole as usize].unit == 0 {
                *in_room.entry(r0.first_spawnpoint + hole - l.units[0].first_spawnpoint).or_default() += 1.0;
            }
        }
        assert_eq!(t.matched, matched, "{name}");
        assert_eq!(t.consistent, matched, "{name}: no facts, so every match agrees");
        assert_eq!(t.hole.spots, in_room, "{name}");
        assert_eq!(t.hole.other, 0.0, "{name}");
        assert!((total(&t) - matched as f64).abs() < 1e-6, "{name}: every layout counted once");
        assert!(!t.hole.exits.is_empty(), "{name}");
        // Every exit is an open door of the pod's unit.
        for d in t.hole.exits.keys() {
            assert_eq!(reference.doors[*d as usize].unit, 0, "{name}");
        }
    }
}

/// Ruling a spot out drops exactly the layouts with the hole there: it gets 0 and the rest keep
/// their counts, so their probabilities are renormalized. Saying the hole is there leaves only it.
#[test]
fn ruling_out_a_spot_renormalizes_the_rest() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    let reference = Layout::generate(0x1234_5678, ci);
    let base = run(&reference, &Observations::default(), ci);
    let (&spot, &n) = base
        .hole
        .spots
        .iter()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .expect("a hole spot in the pod's room");
    assert!(n > 0.0);

    for fact in [Fact::NoHole, Fact::Occupied, Fact::Empty] {
        let obs = Observations {
            seen: vec![],
            facts: vec![(spot, fact)],
        };
        let t = run(&reference, &obs, ci);
        assert_eq!(t.matched, base.matched);
        assert_eq!(t.hole.spots.get(&spot).copied().unwrap_or(0.0), 0.0, "{fact:?}");
        if fact == Fact::NoHole {
            assert_eq!(t.consistent as f64, base.consistent as f64 - n);
            for (k, v) in &base.hole.spots {
                if *k != spot {
                    assert_eq!(t.hole.spots.get(k), Some(v));
                }
            }
            assert_eq!(t.hole.exits, base.hole.exits);
            // As probabilities: p'(x) = p(x) / (1 - p(spot)).
            let p_spot = n / base.consistent as f64;
            for (k, v) in &t.hole.exits {
                let before = base.hole.exits[k] / base.consistent as f64;
                assert!((v / t.consistent as f64 - before / (1.0 - p_spot)).abs() < 1e-9);
            }
        }
    }

    let obs = Observations {
        seen: vec![],
        facts: vec![(spot, Fact::Hole)],
    };
    let t = run(&reference, &obs, ci);
    assert_eq!(t.consistent as f64, n);
    assert_eq!(t.hole.spots.len(), 1);
    assert!(t.hole.exits.is_empty());
}

/// The counts follow the game's weighted pick: in Challenge Mode each empty alcove or dead end
/// gets its unit's sqrt(score) + 10 (the hole score), so averaging each layout's chance for a spot
/// should land near how often the hole was there.
#[test]
fn counts_follow_the_weighted_pick() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    assert!(ci.is_challenge_mode() && !ci.has_geyser && !ci.is_final_floor);
    let reference = Layout::generate(0x1234_5678, ci);
    let n = 6000;
    let t = tally(&reference, &Observations::default(), ci, Generator::Gc, seeds(n), 0);
    let mut expected = std::collections::BTreeMap::<u32, f64>::new();
    for seed in seeds(n) {
        let l = Layout::generate(seed as u32, ci);
        if match_units(&reference, &[0], &l).is_none() {
            continue;
        }
        // Spots that got a hole score were the candidates (the hole's own included).
        let weights: Vec<(usize, f64)> = l
            .spawnpoints
            .iter()
            .enumerate()
            .filter(|(_, sp)| sp.hole_score > 0 && matches!(sp.group(), 4 | 9))
            .map(|(i, sp)| (i, sp.hole_score as f64))
            .collect();
        let sum: f64 = weights.iter().map(|w| w.1).sum();
        assert!(weights.iter().any(|w| w.0 == l.hole.unwrap() as usize));
        for (i, w) in weights {
            if l.spawnpoints[i].unit == 0 {
                *expected
                    .entry(reference.units[0].first_spawnpoint + i as u32 - l.units[0].first_spawnpoint)
                    .or_default() += w / sum;
            }
        }
    }
    let m = t.consistent as f64;
    for (k, e) in &expected {
        let p = e / m;
        let got = t.hole.spots.get(k).copied().unwrap_or(0.0) / m;
        let sd = (p * (1.0 - p) / m).sqrt();
        assert!((got - p).abs() < 4.0 * sd + 1e-3, "spot {k}: counted {got:.4}, weights say {p:.4}");
    }
}

/// A teki seen at a spot means the hole isn't there, and hard teki keep 200 units from the hole,
/// so no consistent layout has the hole that close.
#[test]
fn seen_teki_push_the_hole_away() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    // A reference with a hard teki (group 1) in the pod's unit.
    let (reference, spot) = (0..2000u32)
        .find_map(|s| {
            let l = Layout::generate(s, ci);
            let spot = l
                .unit_spawnpoints(0)
                .iter()
                .position(|sp| sp.group() == 1 && sp.contains.iter().any(|so| matches!(so, SpawnObject::Teki(..))))?;
            Some((l.clone(), l.units[0].first_spawnpoint + spot as u32))
        })
        .expect("a layout with a hard teki in the pod's room");
    let obs = Observations {
        seen: vec![],
        facts: vec![(spot, Fact::Occupied)],
    };
    let t = run(&reference, &obs, ci);
    assert!(t.consistent > 0 && t.consistent < t.matched);
    let pos = reference.spawnpoints[spot as usize].pos;
    for k in t.hole.spots.keys() {
        assert!(reference.spawnpoints[*k as usize].pos.p2_dist(&pos) >= 200.0);
    }
}

/// Observations move to another layout whose seen rooms match, and mean the same there.
#[test]
fn remap_keeps_observations() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    let reference = Layout::generate(0x1234_5678, ci);
    let spot = reference.units[0].first_spawnpoint;
    let obs = Observations {
        seen: vec![0],
        facts: vec![(spot, Fact::NoHole)],
    };
    let same = remap(&reference, &obs, &reference).unwrap();
    assert_eq!(same.seen, vec![0]);
    assert_eq!(same.facts, obs.facts);

    let t = run(&reference, &obs, ci);
    for &seed in &t.examples {
        let other = Layout::generate(seed as u32, ci);
        let moved = remap(&reference, &obs, &other).expect("an example matches");
        let back = remap(&other, &moved, &reference).unwrap();
        assert_eq!(back.facts, obs.facts);
        // The odds are the same seen from either layout.
        let t2 = run(&other, &moved, ci);
        assert_eq!((t2.matched, t2.consistent), (t.matched, t.consistent));
    }
    assert!(remap(&reference, &obs, &Layout::generate(0, caveinfo(&data, "CH1-1"))).is_none());
}

#[test]
fn facts_parse() {
    let obs = Observations::parse("0,3", "12e,40h,7v").unwrap();
    assert_eq!(obs.seen, vec![0, 3]);
    assert_eq!(obs.facts, vec![(12, Fact::Empty), (40, Fact::Hole), (7, Fact::NothingVisible)]);
    assert!(Observations::parse("-", "-").unwrap().facts.is_empty());
    assert!(Observations::parse("x", "-").is_err());
    assert!(Observations::parse("-", "12q").is_err());
}

// ---- Odds from one layout's own weights (`weighted`) ----

/// The layout as it was when the hole was picked (unit scores before teki).
fn at_exits(seed: u32, ci: &CaveInfo) -> Layout<'_> {
    generate_partly(Rng::new(seed), ci, None, true).unwrap()
}

fn all_seen(l: &Layout) -> Observations {
    Observations {
        seen: (0..l.units.len() as u32).collect(),
        facts: vec![],
    }
}

/// The candidates and weights are the ones the generator used: in Challenge Mode the spots that
/// got a hole score, with it as the weight; the hole and geyser always land on one.
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
            for exit in [l.hole, l.geyser].into_iter().flatten() {
                assert!(cands.iter().any(|c| c.0 == exit && c.1 > 0.0), "{name} {seed:#X}");
            }
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

/// With every room seen, each spot gets weight / total; ruling one out renormalizes the rest
/// exactly; saying the hole is somewhere gives it 1.
#[test]
fn weighted_conditioning() {
    let data = GameData::bundled().unwrap();
    let ci = caveinfo(&data, "CH12-1");
    let l = at_exits(0x1234_5678, ci);
    let obs = all_seen(&l);
    let cands = exit_candidates(&l);
    let total: f64 = cands.iter().map(|c| c.1).sum();
    let t = weighted(&l, &obs);
    assert_eq!(t.consistent, 1);
    assert!(t.hole.exits.is_empty());
    for &(i, w) in &cands {
        assert!((t.hole.spots[&i] - w / total).abs() < 1e-12);
    }
    let (out, w_out) = cands[0];
    let mut ruled = obs.clone();
    ruled.facts.push((out, Fact::NoHole));
    let t2 = weighted(&l, &ruled);
    assert!(!t2.hole.spots.contains_key(&out));
    for &(i, w) in &cands[1..] {
        assert!((t2.hole.spots[&i] - w / (total - w_out)).abs() < 1e-12);
    }
    let mut here = obs.clone();
    here.facts.push((out, Fact::Hole));
    let t3 = weighted(&l, &here);
    assert_eq!(t3.hole.spots.len(), 1);
    assert!((t3.hole.spots[&out] - 1.0).abs() < 1e-12);
    // Ruling out every spot leaves nothing consistent.
    let mut none = obs.clone();
    none.facts.extend(cands.iter().map(|c| (c.0, Fact::Empty)));
    assert_eq!(weighted(&l, &none).consistent, 0);
    // Only the pod's room seen: the rest goes to its exits, and it all sums to 1.
    let t4 = weighted(&l, &Observations::default());
    let sum: f64 = t4.hole.spots.values().chain(t4.hole.exits.values()).sum::<f64>() + t4.hole.other;
    assert!((sum - 1.0).abs() < 1e-9);
    assert!(t4.hole.exits.keys().all(|&d| l.doors[d as usize].unit == 0));
}

/// The weights are calibrated against real generation: over many seeds, the chance each layout
/// gave its actual hole spot averages to the sum of its squared chances (what it should be if the
/// odds are right). Also for the geyser on a final floor, and on a floor with both, where the
/// geyser's odds depend on where the hole went.
#[test]
fn weighted_odds_are_calibrated() {
    let data = GameData::bundled().unwrap();
    let both = data
        .caves()
        .iter()
        .filter(|c| c.is_challenge_mode && c.game == "pikmin2")
        .flat_map(|c| data.floors(c).unwrap().iter())
        .find(|ci| ci.has_geyser && !ci.is_final_floor)
        .map(|ci| ci.name());
    let mut names = vec!["CH12-1".to_string(), "CH5-2".to_string(), "CH20-1".to_string()];
    names.extend(both);
    for name in names {
        let ci = caveinfo(&data, &name);
        let (mut hit, mut expect, mut var) = ([0.0; 2], [0.0; 2], [0.0; 2]);
        for seed in seeds(3000) {
            let l = at_exits(seed as u32, ci);
            let t = weighted(&l, &all_seen(&l));
            for (k, (w, actual)) in [(&t.hole, l.hole), (&t.geyser, l.geyser)].into_iter().enumerate() {
                let Some(actual) = actual else { continue };
                let sq: f64 = w.spots.values().map(|p| p * p).sum();
                let p = w.spots.get(&actual).copied().unwrap_or(0.0);
                assert!(p > 0.0, "{name} {seed:#X}: the real spot had no chance");
                hit[k] += p;
                expect[k] += sq;
                var[k] += w.spots.values().map(|p| p * p * p).sum::<f64>() - sq * sq;
            }
        }
        for k in 0..2 {
            if expect[k] > 0.0 {
                let sd = var[k].max(0.0).sqrt();
                assert!(
                    (hit[k] - expect[k]).abs() < 5.0 * sd + 1e-6,
                    "{name} {k}: {:.2} vs {:.2} (sd {sd:.2})",
                    hit[k],
                    expect[k]
                );
            }
        }
    }
}
