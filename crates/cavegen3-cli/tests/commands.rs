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
    let text = cavegen3(&["count", "SCx-7", "minihoudai", "-n", "5000", "--seeds", "c", "--start", "0"]);
    let shares: f64 = text
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1)?.strip_suffix('%')?.parse::<f64>().ok())
        .sum();
    assert!((shares - 100.0).abs() < 0.1, "{text}");
    // Seeds with fewer than two Gatling Groinks are the "1" row of the count.
    let stats = cavegen3(&["stats", "SCx-7", "minihoudai < 2", "-n", "5000", "--seeds", "c", "--start", "0"]);
    let matched: u64 = stats.lines().nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
    let one_row = text.lines().find(|l| l.split_whitespace().next() == Some("1")).map_or(0.0, |l| {
        l.split_whitespace().nth(1).unwrap().trim_end_matches('%').parse::<f64>().unwrap()
    });
    assert!((matched as f64 / 50.0 - one_row).abs() < 0.02, "{stats}\n{text}");
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

#[test]
fn colossal_options_reach_the_generator() {
    let dir = std::env::temp_dir().join("cavegen3-colossal-test");
    let out = cavegen3(&["colossal+5onion", "0", "--scale", "0.1", "-o", dir.to_str().unwrap()]);
    assert!(out.contains("colossal1_5onion_00000000.png"), "{out}");
    let err = Command::new(env!("CARGO_BIN_EXE_cavegen3")).args(["SCx-6+ultra", "0"]).output().unwrap();
    assert!(!err.status.success());
    assert!(String::from_utf8_lossy(&err.stderr).contains("only Colossal Caverns"));
}
