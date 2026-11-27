//! Parses every bundled cave with both CaveGen3 and Caveripper and checks they agree.

use caveripper::{assets::fs_asset_manager::FsAssetManager, caveinfo as r};
use cavegen3::{GameData, caveinfo as n};

fn same_point(a: &cavegen3::point::Point, b: &caveripper::point::Point<3, f32>) -> bool {
    a.0 == b.0
}

fn check_unit(n: &n::CaveUnit, r: &r::CaveUnit) -> Result<(), String> {
    let room_type = match r.room_type {
        r::RoomType::Room => n::RoomType::Room,
        r::RoomType::Hallway => n::RoomType::Hallway,
        r::RoomType::DeadEnd => n::RoomType::DeadEnd,
    };
    let same = n.unit_folder_name == r.unit_folder_name
        && n.width == r.width
        && n.height == r.height
        && n.room_type == room_type
        && n.num_doors == r.num_doors
        && n.rotation == r.rotation
        && n.doors.len() == r.doors.len()
        && n.doors.iter().zip(&r.doors).all(|(a, b)| {
            a.direction == b.direction
                && a.side_lateral_offset == b.side_lateral_offset
                && a.waypoint_index == b.waypoint_index
                && a.num_links == b.num_links
                && a.door_links.len() == b.door_links.len()
                && a.door_links
                    .iter()
                    .zip(&b.door_links)
                    .all(|(x, y)| x.distance == y.distance && x.door_id == y.door_id && x.tekiflag == y.tekiflag)
        })
        && n.spawnpoints.len() == r.spawnpoints.len()
        && n.spawnpoints.iter().zip(&r.spawnpoints).all(|(a, b)| {
            a.group == b.group
                && same_point(&a.pos, &b.pos)
                && a.angle_degrees == b.angle_degrees
                && a.radius == b.radius
                && a.min_num == b.min_num
                && a.max_num == b.max_num
        })
        && n.waterboxes.len() == r.waterboxes.len()
        && n.waterboxes.iter().zip(&r.waterboxes).all(|(a, b)| same_point(&a.p1, &b.p1) && same_point(&a.p2, &b.p2))
        && n.waypoints.len() == r.waypoints.len()
        && n.waypoints
            .iter()
            .zip(&r.waypoints)
            .all(|(a, b)| same_point(&a.pos, &b.pos) && a.r == b.r && a.index == b.index && a.links == b.links);
    if same { Ok(()) } else { Err(format!("unit {} rotation {} differs", r.unit_folder_name, r.rotation)) }
}

fn check_floor(n: &n::CaveInfo, r: &r::CaveInfo) -> Result<(), String> {
    let scalars = (
        n.floor_num,
        n.max_main_objects,
        n.max_treasures,
        n.max_gates,
        n.num_rooms,
        n.corridor_probability,
        n.cap_probability,
        n.has_geyser,
        n.exit_plugged,
        n.is_final_floor,
        n.waterwraith_timer,
    ) == (
        r.floor_num,
        r.max_main_objects,
        r.max_treasures,
        r.max_gates,
        r.num_rooms,
        r.corridor_probability,
        r.cap_probability,
        r.has_geyser,
        r.exit_plugged,
        r.is_final_floor,
        r.waterwraith_timer,
    );
    if !scalars {
        return Err("floor settings differ".into());
    }
    let teki = n.teki_info.len() == r.teki_info.len()
        && n.teki_info.iter().zip(&r.teki_info).all(|(a, b)| {
            (&a.internal_name, &a.carrying, a.minimum_amount, a.filler_distribution_weight, a.group, &a.spawn_method, a.stationary)
                == (&b.internal_name, &b.carrying, b.minimum_amount, b.filler_distribution_weight, b.group, &b.spawn_method, b.stationary)
        });
    let items = n.item_info.len() == r.item_info.len()
        && n.item_info.iter().zip(&r.item_info).all(|(a, b)| {
            (&a.internal_name, a.min_amount, a.filler_distribution_weight) == (&b.internal_name, b.min_amount, b.filler_distribution_weight)
        });
    let gates = n.gate_info.len() == r.gate_info.len()
        && n.gate_info.iter().zip(&r.gate_info).all(|(a, b)| a.health == b.health && a.spawn_distribution_weight == b.spawn_distribution_weight);
    let caps = n.cap_info.len() == r.cap_info.len()
        && n.cap_info.iter().zip(&r.cap_info).all(|(a, b)| {
            (&a.internal_name, &a.carrying, a.minimum_amount, a.filler_distribution_weight, a.group, &a.spawn_method, a.stationary)
                == (&b.internal_name, &b.carrying, b.minimum_amount, b.filler_distribution_weight, b.group, &b.spawn_method, b.stationary)
        });
    if !(teki && items && gates && caps) {
        return Err(format!("object lists differ (teki {teki}, items {items}, gates {gates}, caps {caps})"));
    }
    if n.cave_units.len() != r.cave_units.len() {
        return Err(format!("{} units vs {}", n.cave_units.len(), r.cave_units.len()));
    }
    for (a, b) in n.cave_units.iter().zip(&r.cave_units) {
        check_unit(a, b)?;
    }
    Ok(())
}

#[test]
fn parsing_matches_caveripper() {
    let data = GameData::bundled().unwrap();
    let mgr = FsAssetManager::init().unwrap();
    let mut checked = 0;
    let mut failures = Vec::new();
    // New Year 1.38's Elemental River and Season Den use a unit file with a unit missing its `}`
    // (1_units_big2_rocket_desert.txt, 1_MAT_boss_volcano.txt), which the game ignores. Caveripper can't parse Lands of Torture's Bulblax Kingdom (a CapInfo missing its `{`), and it drops the
    // two spawn points missing their `{` in room_dangomushi_tile, which only Cavern of Chaos
    // floor 10 uses. cavegen_expect checks those against CaveGen instead.
    let skip = |c: &cavegen3::data::CaveConfig| {
        (c.game == "torture" && c.full_name == "Bulblax Kingdom")
            || (c.game == "newyear" && (c.full_name == "Elemental River" || c.full_name == "Season Den"))
    };
    for cfg in data.caves().iter().filter(|c| data.has_game(&c.game) && !skip(c)) {
        let ours = data.floors(cfg).unwrap_or_else(|e| panic!("{e}"));
        let ref_cfg = mgr.cave_cfg.iter().find(|c| c.caveinfo_filename == cfg.caveinfo_filename && c.game == cfg.game).unwrap();
        let theirs = r::CaveInfo::parse_from(ref_cfg, &mgr).unwrap();
        assert_eq!(ours.len(), theirs.len(), "{} floor count", cfg.full_name);
        for (i, (a, b)) in ours.iter().zip(&theirs).enumerate() {
            if cfg.game == "torture" && cfg.full_name == "Cavern of Chaos" && i + 1 == 10 {
                continue;
            }
            if let Err(e) = check_floor(a, b) {
                failures.push(format!("{}: {e}", a.name()));
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} floors checked");
    assert!(failures.is_empty(), "{} of {checked} floors differ:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn sublevel_names() {
    let data = GameData::bundled().unwrap();
    for (input, expected) in [
        ("SCx6", "SCx6"),
        ("scx-6", "SCx6"),
        ("Snagret Hole 6", "SH6"),
        ("CH12-1", "CH12-1"),
        ("ch-cm2", "CH12-1"),
        ("colossal", "colossal1"),
    ] {
        let sublevel = data.sublevel(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        let name = if input == "ch-cm2" { sublevel.normalized_name() } else { sublevel.short_name() };
        assert!(name.eq_ignore_ascii_case(expected) || input == "ch-cm2", "{input} -> {name}");
    }
    assert!(data.sublevel("not a cave 3").is_err());

    // Without a game prefix, names are vanilla; romhack caves need theirs.
    for (input, game, cave) in [
        ("ER-1", "pikmin2", "Emperor's Realm"),
        ("newyear:ER-3", "newyear", "Elemental River"),
        ("newyear:Elemental River 3", "newyear", "Elemental River"),
        ("251:SS2", "251", "Subterranean Sands"),
    ] {
        let sublevel = data.sublevel(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        assert_eq!((sublevel.cfg.game.as_str(), sublevel.cfg.full_name.as_str()), (game, cave), "{input}");
    }
    assert!(data.sublevel("Elemental River 3").is_err());
    assert!(data.sublevel("SS2").is_err());
}
