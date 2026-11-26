//! Challenge Mode hole odds (`cavegen3::hole_odds`): the conditioning, checked against direct
//! counts over the same seeds and against the hole's weighted pick.

use cavegen3::{
    GameData, Generator, Layout, SpawnObject,
    caveinfo::CaveInfo,
    hole_odds::{Fact, Observations, Tally, match_units, remap, tally},
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
