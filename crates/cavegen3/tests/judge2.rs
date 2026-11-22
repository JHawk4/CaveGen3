//! judge2 (the refit Pay off Debt judge) against fixed scores from the program it was fitted
//! with. Over 16,000 layouts of the 54 Pay off Debt sublevels matched exactly when it was
//! ported; these few keep it that way. Run with `cargo test -p cavegen3 --test judge2`.

use cavegen3::{
    GameData, Layout,
    jhawk_judge::{JhawkJudge, JudgeType},
};

/// (sublevel, seed, judge2 score, number of take/skip choices, treasures skipped with p > 0.01)
const CASES: &[(&str, u32, f64, usize, &[(&str, f64)])] = &[
    // the lens's carry route crosses water: certainly skipped
    ("SH-6", 0x2B1980E4, 70.83657420840828, 2, &[("bey_goma", 1.0)]),
    // the lens's Bulborb starts in a pool, but its route is dry: taken
    ("SH-6", 0x29698334, 29.262831713071346, 2, &[]),
    ("GK-2", 0x00001234, 54.54409448653781, 2, &[("g_futa_kyusyu", 0.256)]),
    ("GK-3", 0x5A6727F3, 52.77499927623664, 4, &[]),
    ("SCx-7", 0x1234ABCD, 76.6036949263671, 8, &[("tape_red", 0.084)]),
    ("BK-4", 0x1234ABCD, 51.2285574245525, 4, &[]),
    ("EC-1", 0x00000007, 43.5076992252432, 4, &[]),
    ("WFG-1", 0xDEADBEEF, 49.88761932484975, 2, &[("fire_helmet", 0.573)]),
    ("FC-6", 0x0BADF00D, 21.21476375579834, 1, &[]),
];

#[test]
fn judge2_matches_its_reference() {
    let data = GameData::bundled().expect("Couldn't load the bundled game data");
    for &(sub, seed, expected, num_choices, skips) in CASES {
        let sublevel = data.sublevel(sub).unwrap();
        let caveinfo = data.caveinfo(&sublevel).unwrap();
        let judge = JhawkJudge::new(JudgeType::Judge2, caveinfo);
        let layout = Layout::generate(seed, caveinfo);
        let r = judge.judge2(&layout);
        assert!((r.score - expected).abs() < 1e-9, "{sub} {seed:08X}: {} vs {expected}", r.score);
        assert_eq!(judge.judge(&layout), r.score, "{sub} {seed:08X}");
        assert_eq!(r.choices.len(), num_choices, "{sub} {seed:08X}");
        assert!(r.choices[0].skipped.is_empty());
        let p: f64 = r.choices.iter().map(|c| c.p).sum();
        assert!((p - 1.0).abs() < 1e-9);
        let likely: Vec<(String, f64)> = r.skip_chances().into_iter().filter(|(_, p)| *p > 0.01).collect();
        assert_eq!(likely.len(), skips.len(), "{sub} {seed:08X}: {likely:?}");
        for ((name, p), (en, ep)) in likely.iter().zip(skips) {
            assert_eq!(name, en, "{sub} {seed:08X}");
            assert!((p - ep).abs() < 0.001, "{sub} {seed:08X}: {name} {p} vs {ep}");
        }
    }
}

#[test]
fn judge2_names() {
    assert_eq!("judge2".parse::<JudgeType>(), Ok(JudgeType::Judge2));
    assert_eq!("pod2".parse::<JudgeType>(), Ok(JudgeType::Judge2));
    assert_eq!(JudgeType::Judge2.to_string(), "judge2");
    assert!(JudgeType::ALL.contains(&JudgeType::Judge2));
}
