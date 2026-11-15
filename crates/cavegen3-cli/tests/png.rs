//! The command-line tool's images: sizes, contents and every game's pictures.

use cavegen3_cli::{
    Painter, default_data_dir,
    render::{CELL, OPTION_NAMES, Options},
};
use tiny_skia::Pixmap;

fn painter() -> Painter {
    Painter::new(&default_data_dir()).unwrap()
}

#[test]
fn whole_map_png() {
    let p = painter();
    let (layout, png) = p.png("SCx-6", 0x1234ABCD, &Options::default()).unwrap();
    assert_eq!(layout.seed, "1234ABCD");
    let img = Pixmap::decode_png(&png).unwrap();
    // The cave plus half a cell on each side.
    assert_eq!(img.width(), ((layout.width + 1.0) * CELL) as u32);
    assert_eq!(img.height(), ((layout.height + 1.0) * CELL) as u32);
    // Mostly background, but with map units drawn.
    let bright = img.pixels().iter().filter(|c| c.red() > 150).count();
    assert!(bright > img.pixels().len() / 10, "only {bright} bright pixels");
}

#[test]
fn comedown_and_scale() {
    let p = painter();
    let opts = Options {
        comedown: true,
        scale: 2.0,
        ..Options::default()
    };
    let (_, png) = p.png("BK-1", 7, &opts).unwrap();
    let img = Pixmap::decode_png(&png).unwrap();
    // About as wide as deep at 16:9; the corners are outside the trapezoid, so background.
    assert!(
        img.width().abs_diff(img.height()) < img.width() / 5,
        "{}x{}",
        img.width(),
        img.height()
    );
    let corner = img.pixel(1, 1).unwrap();
    assert_eq!((corner.red(), corner.green(), corner.blue()), (15, 15, 15));
}

#[test]
fn every_option_draws() {
    let p = painter();
    let mut opts = Options::default();
    for name in OPTION_NAMES {
        opts.set(name, true).unwrap();
    }
    assert!(opts.needs_details());
    for sublevel in ["FC-3", "SCx-7", "CH12-1", "colossal"] {
        p.png(sublevel, 99, &opts).unwrap_or_else(|e| panic!("{sublevel}: {e}"));
    }
    assert!(opts.set("nonsense", true).is_err());
    assert!(opts.set("door-scores", false).is_ok() && !opts.door_scores);
}

/// Every map unit on the first floor of every cave of every game has its radar image, and every
/// object an icon (a few romhack teki and treasures have none and get a lettered disc).
#[test]
fn every_game_has_its_images() {
    let p = painter();
    let data = cavegen3::GameData::from_dir(default_data_dir()).unwrap();
    let mut missing_icons = Vec::new();
    // New Year's Timeless Realm needs a unit file the romhack doesn't ship (the other tests skip it too).
    for cave in data
        .caves()
        .iter()
        .filter(|c| data.has_game(&c.game) && !(c.game == "newyear" && c.full_name == "Timeless Realm"))
    {
        let name = format!("{}:{}-1", cave.game, cave.short_name());
        let layout = p.layout(&name, 1, false).unwrap_or_else(|e| panic!("{name}: {e}"));
        for u in &layout.units {
            assert!(p.assets.unit(&u.name, &u.game).is_some(), "{name}: no image for unit {}", u.name);
        }
        for o in &layout.objects {
            let (n, g) = (o.name.as_deref().unwrap_or(""), o.game.as_deref().unwrap_or("pikmin2"));
            let found = match o.kind.as_str() {
                "teki" => p.assets.teki(n, g).is_some(),
                "treasure" => p.assets.treasure(n, g).is_some(),
                _ => true,
            };
            if !found {
                missing_icons.push(format!("{g}/{n}"));
            }
        }
    }
    missing_icons.sort();
    missing_icons.dedup();
    // Vanilla Pikmin 2 has every icon.
    assert!(
        !missing_icons.iter().any(|m| m.starts_with("pikmin2/")),
        "missing: {missing_icons:?}"
    );
    for name in [
        "pod_icon",
        "Cave_icon",
        "36px-Clog_icon",
        "Geyser_icon",
        "Gray_bramble_gate_icon",
        "falling_icon",
        "onion0",
    ] {
        assert!(p.assets.special(name).is_some(), "no {name}");
    }
}
