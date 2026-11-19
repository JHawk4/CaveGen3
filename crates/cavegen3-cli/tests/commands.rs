//! The command-line tool's commands, run as a user would.

use std::process::Command;

fn cavegen3(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cavegen3")).args(args).output().expect("couldn't run cavegen3");
    assert!(out.status.success(), "cavegen3 {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn caveinfo_lists_what_the_floor_has() {
    let text = cavegen3(&["caveinfo", "SC-2"]);
    assert!(text.contains("Submerged Castle 2"));
    assert!(text.contains("Waterwraith (BlackMan), carrying Professional Noisemaker"));
    assert!(text.contains("Chocolate Cushion (donutschoco)"));
    assert!(text.contains("Waterwraith 300 s"));
}

#[test]
fn judge_scores_match_the_search() {
    let one = cavegen3(&["judge", "BK-4", "pod", "1234ABCD"]);
    assert!(one.starts_with("1234ABCD  "), "{one}");
    // A one-seed search starting there finds the same score.
    let found = cavegen3(&["search", "BK-4", "--judge", "pod", "-n", "1", "--seeds", "c", "--start", "1234ABCD", "--json"]);
    let score: f64 = one.split_whitespace().nth(1).unwrap().parse().unwrap();
    let json: serde_json::Value = serde_json::from_str(&found).unwrap();
    assert_eq!(json["best"][0]["seed"], "1234ABCD");
    assert_eq!(json["best"][0]["score"].as_f64().unwrap(), score);
}

#[test]
fn stats_and_count_add_up() {
    // Rows: how many, seeds with that many, "seeds", share.
    let text = cavegen3(&["count", "SCx-7", "minihoudai", "-n", "5000", "--seeds", "c", "--start", "0"]);
    let rows: Vec<(u64, u64)> = text
        .lines()
        .filter_map(|l| {
            let mut w = l.split_whitespace();
            let (k, n, unit) = (w.next()?.parse().ok()?, w.next()?.parse().ok()?, w.next()?);
            (unit == "seeds").then_some((k, n))
        })
        .collect();
    assert_eq!(rows.iter().map(|(_, n)| n).sum::<u64>(), 5000, "{text}");
    // Seeds with fewer than two Gatling Groinks are exactly the "1" row of the count.
    let stats = cavegen3(&["stats", "SCx-7", "minihoudai < 2", "-n", "5000", "--seeds", "c", "--start", "0"]);
    let matched: u64 = stats.lines().nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    let ones = rows.iter().filter(|(k, _)| *k < 2).map(|(_, n)| n).sum::<u64>();
    assert_eq!(matched, ones, "{stats}\n{text}");
}

#[test]
fn game_rng_seeds_continue_across_slices() {
    // 5000 game-RNG seeds span two of the search's parallel slices; listing every seed checks
    // that the second slice continues the RNG rather than restarting it.
    let json: serde_json::Value = serde_json::from_str(&cavegen3(&[
        "search", "EC-1", "--count", "treasure", "-n", "5000", "--seeds", "g", "--start", "0", "--top", "5000", "--json",
    ]))
    .unwrap();
    let seeds: Vec<u32> =
        json["best"].as_array().unwrap().iter().map(|b| u32::from_str_radix(b["seed"].as_str().unwrap(), 16).unwrap()).collect();
    assert_eq!(seeds.len(), 5000);
    let mut s = 0u32;
    for (i, &seed) in seeds.iter().enumerate() {
        assert_eq!(seed, s, "seed {i}");
        s = s.wrapping_mul(0x41c6_4e6d).wrapping_add(0x3039);
    }
}

#[test]
fn aggregate_writes_group_images() {
    let dir = std::env::temp_dir().join("cavegen3-agg-test");
    let _ = std::fs::remove_dir_all(&dir);
    let text = cavegen3(&["aggregate", "HoB-2", "-n", "2000", "--seeds", "c", "--start", "0", "--max", "3", "-o", dir.to_str().unwrap()]);
    assert!(text.contains("2000 seeds of HoB-2"), "{text}");
    let pngs = std::fs::read_dir(&dir).unwrap().count();
    assert_eq!(pngs, 3);
}

/// Grouping by the comedown view: the comedown camera sees more of the floor than the landing
/// camera, so it splits the same seeds into at least as many groups.
#[test]
fn aggregate_by_comedown_view() {
    let groups = |group: &str| -> usize {
        let dir = std::env::temp_dir().join(format!("cavegen3-agg-{group}-test"));
        let _ = std::fs::remove_dir_all(&dir);
        let text = cavegen3(&[
            "aggregate", "HoB-2", "--group", group, "-n", "2000", "--seeds", "c", "--start", "0", "--max", "2", "-o",
            dir.to_str().unwrap(),
        ]);
        assert!(text.contains(&format!("_{group}_02.png")), "{text}");
        let line = text.lines().find(|l| l.contains(&format!("groups (by {group})"))).expect("summary line");
        line.split(", ").nth(1).and_then(|s| s.split(' ').next()).unwrap().parse().unwrap()
    };
    assert!(groups("comedown-view") >= groups("landing"));
}

#[test]
fn colossal_options_reach_the_generator() {
    let dir = std::env::temp_dir().join("cavegen3-colossal-test");
    let out = cavegen3(&["colossal+5onion", "0", "--scale", "0.1", "-o", dir.to_str().unwrap()]);
    assert!(out.contains("colossal1_5onion_00000000.png"), "{out}");
    let err = Command::new(env!("CARGO_BIN_EXE_cavegen3")).args(["SCx-6+ultra", "0"]).output().unwrap();
    assert!(!err.status.success());
    assert!(String::from_utf8_lossy(&err.stderr).contains("only Colossal Caverns"));
}
