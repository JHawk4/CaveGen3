//! Checks queries give the same answers as Caveripper's, and keeps its fixed-seed query tests.

use caveripper::{
    assets::fs_asset_manager::FsAssetManager,
    query::{Query as RefQuery, StructuralQuery as RefStructuralQuery},
};
use cavegen3::{
    GameData,
    query::{Query, StructuralQuery},
};
use rayon::prelude::*;

/// One query of every kind, on a mix of floors.
const QUERIES: &[&str] = &[
    "scx7 minihoudai < 2",
    "gk3 castanets = 0",
    "bk4 gate = 1",
    "sh6 hole carry dist < 1500",
    "fc3 toy_ring_a_green carry dist < 600",
    "sr7 any straight dist ship < 400",
    "cos4 hole not gated",
    "sh7 hole gated",
    "scx8 hallway > 30",
    "cos1 alcove > 6",
    "fc2 room_saka1_1_snow = 2",
    "sh6 room_4x4f_4_conc + ship -> room_4x4g_4_conc + fuefuki -> room_4x4b_4_conc -> alcove + hole",
    "fc2 any + ship -> any + toy_ring_c_blue",
    "bk4 kochappy > 4 & sh6 gate = 0",
    "cm1 hole carry dist < 1200",
];

#[test]
fn queries_match_reference() {
    let data = GameData::bundled().unwrap();
    let mgr = FsAssetManager::init().unwrap();
    let results: Vec<Result<bool, String>> = QUERIES
        .par_iter()
        .map(|q| {
            let ours = StructuralQuery::try_parse(q, &data).unwrap_or_else(|e| panic!("{q}: {e}"));
            let theirs = RefStructuralQuery::try_parse(q, &mgr).unwrap_or_else(|e| panic!("{q}: {e}"));
            let seeds: u32 = 400;
            let mut matches = 0;
            for seed in 0..seeds {
                let (a, b) = (ours.matches(seed, &data), theirs.matches(seed, &mgr));
                if a != b {
                    return Err(format!("{q}: seed {seed:#010X}: ours {a}, Caveripper {b}"));
                }
                matches += a as u32;
            }
            // Whether the query varies between seeds, so it actually tests something.
            Ok(matches > 0 && matches < seeds)
        })
        .collect();
    let failures: Vec<&str> = results.iter().filter_map(|r| r.as_ref().err().map(|s| s.as_str())).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let varying = results.iter().filter(|r| matches!(r, Ok(true))).count();
    assert!(varying * 3 >= QUERIES.len() * 2, "only {varying} of {} queries vary between seeds", QUERIES.len());
}

fn check(query: &str, matching: &[u32], not_matching: &[u32]) {
    let data = GameData::bundled().unwrap();
    let q = StructuralQuery::try_parse(query, &data).unwrap_or_else(|e| panic!("{query}: {e}"));
    for s in matching {
        assert!(q.matches(*s, &data), "{query} should match {s:#010X}");
    }
    for s in not_matching {
        assert!(!q.matches(*s, &data), "{query} shouldn't match {s:#010X}");
    }
}

// The following are Caveripper's own query tests.

#[test]
fn towerless() {
    check(
        "scx7 minihoudai < 2",
        &[0xB5E72294, 0x5A7BED1B, 0x81B6883D, 0xF07C8497, 0xCCA91228, 0x69038F7C, 0xDAD4B7A9, 0xF9230A5C, 0xDF0ABBDC, 0xC419B9C9],
        &[0x42AC4C0F, 0x3E6026DE, 0x26834113, 0xF26DA583, 0x036B3F40, 0xFD37369A, 0x5ED119C1, 0x14CAFF6E, 0x906DC0EE, 0xDDD13F62],
    );
}

#[test]
fn room_path() {
    check(
        "sh6 room_4x4f_4_conc + ship -> room_4x4g_4_conc + bluekochappy/bey_goma + fuefuki -> room_4x4b_4_conc -> alcove + hole",
        &[0x17531C52, 0x7A1B9265, 0x2178B525, 0x19775101, 0x7E6568A2, 0xB98790CE, 0xE366862F, 0x7F9E8E7F, 0x73A52E53, 0x7B5058DF],
        &[0x42AC4C0F, 0x3E6026DE, 0x26834113, 0xF26DA583, 0x036B3F40, 0xFD37369A, 0x5ED119C1, 0x14CAFF6E, 0x906DC0EE, 0xDDD13F62],
    );
}

#[test]
fn clackerless() {
    check(
        "gk3 castanets = 0",
        &[0x3D10D570, 0xF5F4D7A8, 0x950A49A3, 0x072BDE2E, 0x9D1F0152, 0xE3EF8C67, 0xA45CE0BA, 0xA8DF21A4, 0x16968C0D, 0xA5D15522],
        &[0x0407B6C5, 0xEA493EAC, 0xA92697B2, 0xFFBF35A8, 0x31A9BEFF, 0x732B700C, 0x505282B2, 0x240E934C, 0x23BBEA60, 0xD9D7CC12],
    );
}

#[test]
fn not_gated() {
    check(
        "cos4 hole not gated",
        &[0x3217B696, 0xEBCBACA6, 0x676DBC71, 0x31FE7AD3, 0x439AC19F, 0x140784F1, 0x10C90541, 0xD5CE0227, 0xDE71515F, 0xBF9A701A],
        &[0xD975FE4B, 0xCB1BC5D5, 0x4AE3BF7D, 0xDD5F6117, 0xB5E1865C, 0x120BEC31, 0xEC30A1E8, 0x910B7612, 0xD766A798, 0x0654F60C],
    );
}

#[test]
fn parses_query_forms() {
    let data = GameData::bundled().unwrap();
    for q in [
        "scx8 hallway > 50",
        "scx8 hall > 50",
        "cos1 cap > 10",
        "cos1 alcove > 10",
        "sr7 room < 2",
        "fc2 room_saka1_1_snow = 2",
        "scx7 room_ari1_3_metal < 2",
        "bd1 geyser = 0",
        "fc2 any+ship->any+toy_ring_c_blue",
        "fc2 any + ship -> any + toy_ring_c_blue",
        "fc2 any    +    ship   ->    any +        toy_ring_c_blue",
    ] {
        StructuralQuery::try_parse(q, &data).unwrap_or_else(|e| panic!("{q}: {e}"));
    }
    assert!(StructuralQuery::try_parse("sh6 notarealthing > 1", &data).is_err());
}
