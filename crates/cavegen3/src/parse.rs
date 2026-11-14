//! Parsing the game's brace-delimited text files: caveinfo, unit files, and each unit's
//! layout, route and waterbox files.
//!
//! The field layouts follow Caveripper's parser (MIT, https://github.com/mayabyte/caveripper).
//! Caveripper used a pest grammar; this is a small hand-written tokenizer that produces the
//! same sections and lines.

use std::str::FromStr;

use crate::{
    caveinfo::{CapInfo, CaveInfo, CaveUnit, DoorLink, DoorUnit, GateInfo, ItemInfo, RoomType, SpawnPoint, TekiInfo, Waterbox, Waypoint},
    data::{CaveConfig, GameData},
    error::{Error, Result},
    point::Point,
};

/// One `{ ... }` block.
#[derive(Debug, Clone, Default)]
pub(crate) struct Section<'a> {
    pub lines: Vec<Line<'a>>,
}

/// One non-empty line of a section. A leading `{xxxx}` item is split out as the tag.
#[derive(Debug, Clone, Default)]
pub(crate) struct Line<'a> {
    pub tag: Option<&'a str>,
    pub items: Vec<&'a str>,
}

/// Splits a file into sections. Comments run from `#` to the end of the line. Text between
/// sections (such as the count before each `{`, stray `}`s or trailing garbage) is ignored.
pub(crate) fn sections(text: &str) -> Vec<Section<'_>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'#' => i = skip_comment(bytes, i),
            b'{' => {
                let (section, end) = read_section(text, i + 1);
                out.push(section);
                i = end;
            }
            _ => i += 1,
        }
    }
    out
}

/// Skips to the end of the line. A lone `\r` ends a line too, as in Caveripper's grammar. This
/// matters for Colossal Caverns, whose unit file has a `}` between a comment's `\r` and `\n`.
fn skip_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && !matches!(bytes[i], b'\n' | b'\r') {
        i += 1;
    }
    i
}

fn is_separator(b: u8) -> bool {
    matches!(b, b' ' | b'\t')
}

/// Reads lines until the closing `}`. Returns the section and the index after the `}`.
fn read_section(text: &str, mut i: usize) -> (Section<'_>, usize) {
    let bytes = text.as_bytes();
    let mut section = Section::default();
    let mut line: Vec<&str> = Vec::new();
    while i < bytes.len() {
        match bytes[i] {
            b'\n' | b'\r' => {
                finish_line(&mut line, &mut section);
                i += 1;
            }
            b'#' => i = skip_comment(bytes, i),
            b if is_separator(b) => i += 1,
            b'}' => {
                finish_line(&mut line, &mut section);
                return (section, i + 1);
            }
            b'{' => {
                // A tag such as {f000}.
                let start = i;
                while i < bytes.len() && !matches!(bytes[i], b'}' | b'\n' | b'\r') {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'}' {
                    i += 1;
                }
                line.push(&text[start..i]);
            }
            _ => {
                let start = i;
                while i < bytes.len() && !is_separator(bytes[i]) && !matches!(bytes[i], b'\n' | b'\r' | b'#' | b'{' | b'}') {
                    i += 1;
                }
                line.push(&text[start..i]);
            }
        }
    }
    finish_line(&mut line, &mut section);
    (section, i)
}

/// Adds the collected items as a line, splitting off a leading `{xxxx}` tag.
fn finish_line<'a>(line: &mut Vec<&'a str>, section: &mut Section<'a>) {
    if line.is_empty() {
        return;
    }
    let mut items = std::mem::take(line);
    let tag = match items.first() {
        Some(first) if first.len() >= 2 && first.starts_with('{') && first.ends_with('}') => Some(items.remove(0)),
        _ => None,
    };
    section.lines.push(Line { tag, items });
}

impl<'a> Section<'a> {
    pub fn line(&self, index: usize) -> Result<&Line<'a>> {
        self.lines.get(index).ok_or_else(|| Error::new(format!("missing line {index}")))
    }

    /// The value after a tag such as `{f002}`.
    pub fn tag<T: FromStr>(&self, tag: &str) -> Result<T> {
        self.lines
            .iter()
            .find(|line| line.tag.is_some_and(|t| t.eq_ignore_ascii_case(tag)))
            .ok_or_else(|| Error::new(format!("missing tag {tag}")))?
            .item(1)
    }
}

impl Line<'_> {
    /// Parses one item. Items are counted after the tag, but `tag()` asks for item 1 because
    /// tagged lines have a type code before the value.
    pub fn item<T: FromStr>(&self, idx: usize) -> Result<T> {
        let item = self.items.get(idx).ok_or_else(|| Error::new(format!("missing item {idx}")))?;
        item.parse::<T>().or_else(|_| {
            // Some New Year floats are written like "5.6.0000"; the game reads them as "5.6".
            // This also lets integer fields accept values written as "1.000000".
            item.rsplit_once('.')
                .and_then(|(first, _)| first.parse().ok())
                .ok_or_else(|| Error::new(format!("can't parse {item:?} as {}", std::any::type_name::<T>())))
        })
    }
}

/// Parses every floor of a cave.
pub(crate) fn parse_cave(cfg: &CaveConfig, data: &GameData) -> Result<Vec<CaveInfo>> {
    let path = format!("assets/{}/caveinfo/{}", cfg.game, cfg.caveinfo_filename);
    let text = data.read_text(&path)?;
    let all = sections(&text);
    // The first section is the CaveInfo header; then each floor has 5 sections.
    let floors = &all[1.min(all.len())..];
    if floors.is_empty() || floors.len() % 5 != 0 {
        return Err(Error::new(format!("{path}: expected 5 sections per floor, found {}", floors.len())));
    }
    let mut out = floors
        .chunks(5)
        .map(|chunk| parse_floor(cfg, data, chunk).map_err(|e| e.context(&path)))
        .collect::<Result<Vec<_>>>()?;
    out.last_mut().unwrap().is_final_floor = true;
    Ok(out)
}

fn parse_floor(cfg: &CaveConfig, data: &GameData, chunk: &[Section]) -> Result<CaveInfo> {
    let [header, teki, item, gate, cap] = chunk else { unreachable!() };
    let unitfile = if cfg.is_colossal_caverns() { "all_units.txt".to_string() } else { header.tag::<String>("{f008}")? };
    let units = parse_unitfile(&unitfile, cfg, data)?;
    Ok(CaveInfo {
        cave_cfg: cfg.clone(),
        floor_num: header.tag("{f000}")?,
        max_main_objects: header.tag("{f002}")?,
        max_treasures: header.tag("{f003}")?,
        max_gates: header.tag("{f004}")?,
        num_rooms: header.tag("{f005}")?,
        corridor_probability: header.tag("{f006}")?,
        cap_probability: header.tag::<f32>("{f014}")? / 100.0,
        has_geyser: header.tag::<u32>("{f007}")? > 0,
        exit_plugged: header.tag::<u32>("{f010}")? > 0,
        cave_units: expand_rotations(sort_cave_units(units)),
        teki_info: parse_tekiinfo(teki, &cfg.game)?,
        item_info: parse_iteminfo(item, &cfg.game)?,
        gate_info: parse_gateinfo(gate)?,
        cap_info: parse_capinfo(cap, &cfg.game)?,
        is_final_floor: false,
        waterwraith_timer: header.tag("{f016}").unwrap_or(0.0),
    })
}

/// The game's unit sort: like a bubble sort, but when an element is larger than any later
/// element it's moved to the back and the same position is checked again. Not stable.
fn sort_cave_units(mut units: Vec<CaveUnit>) -> Vec<CaveUnit> {
    let mut i = 0;
    while i < units.len() {
        let mut moved = false;
        for j in i + 1..units.len() {
            if units[i].sort_key() > units[j].sort_key() {
                let unit = units.remove(i);
                units.push(unit);
                moved = true;
                break;
            }
        }
        if !moved {
            i += 1;
        }
    }
    units
}

fn expand_rotations(units: Vec<CaveUnit>) -> Vec<CaveUnit> {
    let mut out: Vec<CaveUnit> = units.iter().flat_map(|u| (0..4).map(|r| u.copy_and_rotate_to(r))).collect();
    // Units with the same folder name share a name id.
    let mut names: Vec<&str> = Vec::new();
    let ids: Vec<u16> = out
        .iter()
        .map(|u| match names.iter().position(|n| *n == u.unit_folder_name) {
            Some(i) => i as u16,
            None => {
                names.push(&u.unit_folder_name);
                (names.len() - 1) as u16
            }
        })
        .collect();
    for (unit, id) in out.iter_mut().zip(ids) {
        unit.name_id = id;
    }
    out
}

fn parse_unitfile(unitfile: &str, cfg: &CaveConfig, data: &GameData) -> Result<Vec<CaveUnit>> {
    let path = format!("assets/{}/unitfiles/{unitfile}", cfg.game);
    let text = data.read_text(&path)?;
    sections(&text).iter().map(|s| parse_caveunit(s, cfg, data).map_err(|e| e.context(&path))).collect()
}

fn parse_caveunit(section: &Section, cfg: &CaveConfig, data: &GameData) -> Result<CaveUnit> {
    let name: String = section.line(1)?.item(0)?;
    let width = section.line(2)?.item(0)?;
    let height = section.line(2)?.item(1)?;
    let room_type = RoomType::from_code(section.line(3)?.item(0)?).ok_or_else(|| Error::new(format!("{name}: bad room type")))?;
    let num_doors: usize = section.line(5)?.item(0)?;

    let doors = if num_doors > 0 {
        let per_door = (section.lines.len() - 6) / num_doors;
        section.lines[6..].chunks(per_door).map(parse_door).collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };

    let unit_file = |file: &str| data.read_text(&format!("assets/{}/mapunits/{name}/texts/{file}", cfg.game));

    let mut spawnpoints = match unit_file("layout.txt") {
        Ok(text) => sections(&text).iter().map(parse_spawnpoint).collect::<Result<Vec<_>>>().map_err(|e| e.context(format!("{name}/layout.txt")))?,
        Err(_) => Vec::new(),
    };
    let waterboxes = match unit_file("waterbox.txt") {
        Ok(text) => match sections(&text).first() {
            Some(section) => parse_waterboxes(section).map_err(|e| e.context(format!("{name}/waterbox.txt")))?,
            None => Vec::new(),
        },
        Err(_) => Vec::new(),
    };
    let waypoints = sections(&unit_file("route.txt")?)
        .iter()
        .map(parse_waypoint)
        .collect::<Result<Vec<_>>>()
        .map_err(|e| e.context(format!("{name}/route.txt")))?;

    // Alcoves and hallways can hold the hole or geyser even though their layout files have no
    // spawn point for it. Group 9 stands for that spot; it isn't in the game files.
    if (room_type == RoomType::DeadEnd && name.starts_with("item")) || room_type == RoomType::Hallway {
        spawnpoints.push(SpawnPoint { group: 9, pos: Point::ZERO, angle_degrees: 0.0, radius: 0.0, min_num: 1, max_num: 1 });
    }

    Ok(CaveUnit {
        game: cfg.game.clone(),
        unit_folder_name: name,
        name_id: 0,
        width,
        height,
        room_type,
        num_doors,
        doors,
        rotation: 0,
        spawnpoints,
        waterboxes,
        waypoints,
    })
}

fn parse_door(lines: &[Line]) -> Result<DoorUnit> {
    let line = |i: usize| lines.get(i).ok_or_else(|| Error::new("door definition too short"));
    Ok(DoorUnit {
        direction: line(1)?.item(0)?,
        side_lateral_offset: line(1)?.item(1)?,
        waypoint_index: line(1)?.item(2)?,
        num_links: line(2)?.item(0)?,
        door_links: lines
            .get(3..)
            .unwrap_or_default()
            .iter()
            .map(|l| Ok(DoorLink { distance: l.item(0)?, door_id: l.item(1)?, tekiflag: l.item::<u32>(2)? > 0 }))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn parse_spawnpoint(section: &Section) -> Result<SpawnPoint> {
    Ok(SpawnPoint {
        group: section.line(0)?.item(0)?,
        pos: Point([section.line(1)?.item(0)?, section.line(1)?.item(1)?, section.line(1)?.item(2)?]),
        angle_degrees: section.line(2)?.item(0)?,
        radius: section.line(3)?.item(0)?,
        min_num: section.line(4)?.item(0)?,
        max_num: section.line(5)?.item(0)?,
    })
}

fn parse_waterboxes(section: &Section) -> Result<Vec<Waterbox>> {
    let count: usize = section.line(0)?.item(0)?;
    (1..=count)
        .map(|i| {
            let l = section.line(i)?;
            Ok(Waterbox { p1: Point([l.item(0)?, l.item(1)?, l.item(2)?]), p2: Point([l.item(3)?, l.item(4)?, l.item(5)?]) })
        })
        .collect()
}

fn parse_waypoint(section: &Section) -> Result<Waypoint> {
    let num_links: usize = section.line(1)?.item(0)?;
    let coords = section.line(num_links + 2)?;
    Ok(Waypoint {
        pos: Point([coords.item(0)?, coords.item(1)?, coords.item(2)?]),
        r: coords.item(3)?,
        index: section.line(0)?.item(0)?,
        links: (2..num_links + 2).map(|i| section.line(i)?.item(0)).collect::<Result<Vec<_>>>()?,
    })
}

fn parse_tekiinfo(section: &Section, game: &str) -> Result<Vec<TekiInfo>> {
    section
        .lines
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(2)
        .map(|pair| {
            let identifier: String = pair[0].item(0)?;
            let amount_code: u32 = pair[0].item(1)?;
            let group: u32 = pair[1].item(0)?;
            let id = split_identifier(&identifier);
            // Group 6 (plants) has no weight digit: the whole code is the minimum amount.
            let (minimum_amount, filler_distribution_weight) = if group == 6 { (amount_code, 0) } else { (amount_code / 10, amount_code % 10) };
            Ok(TekiInfo {
                game: game.to_string(),
                internal_name: id.name,
                carrying: id.carrying,
                minimum_amount,
                filler_distribution_weight,
                group,
                spawn_method: id.spawn_method,
                stationary: id.stationary,
            })
        })
        .collect()
}

fn parse_iteminfo(section: &Section, game: &str) -> Result<Vec<ItemInfo>> {
    section
        .lines
        .get(1..)
        .unwrap_or_default()
        .iter()
        .map(|line| {
            let amount_code: u32 = line.item(1)?;
            Ok(ItemInfo {
                game: game.to_string(),
                internal_name: line.item(0)?,
                min_amount: amount_code as u8 / 10,
                filler_distribution_weight: amount_code % 10,
            })
        })
        .collect()
}

fn parse_gateinfo(section: &Section) -> Result<Vec<GateInfo>> {
    section
        .lines
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(2)
        .map(|pair| Ok(GateInfo { health: pair[0].item(1)?, spawn_distribution_weight: pair[1].item(0)? }))
        .collect()
}

fn parse_capinfo(section: &Section, game: &str) -> Result<Vec<CapInfo>> {
    section
        .lines
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(3)
        .map(|triple| {
            let identifier: String = triple[1].item(0)?;
            let amount_code: u32 = triple[1].item(1)?;
            let id = split_identifier(&identifier);
            Ok(CapInfo {
                game: game.to_string(),
                internal_name: id.name,
                carrying: id.carrying,
                minimum_amount: amount_code / 10,
                filler_distribution_weight: amount_code % 10,
                group: triple[2].item(0)?,
                spawn_method: id.spawn_method,
                stationary: id.stationary,
            })
        })
        .collect()
}

struct Identifier {
    spawn_method: Option<String>,
    name: String,
    carrying: Option<String>,
    stationary: bool,
}

/// Splits a combined identifier such as `$1Kochappy_ahiru` into spawn method "1", teki
/// "Kochappy" and carried treasure "ahiru". The teki name is the first `_`-separated part
/// plus any following parts that start with a capital letter or are "s" or "l".
fn split_identifier(combined: &str) -> Identifier {
    let (spawn_method, rest) = match combined.find('$') {
        Some(pos) => {
            let after = &combined[pos + 1..];
            let digit_len = after.chars().next().filter(char::is_ascii_digit).map_or(0, |_| 1);
            (Some(after[..digit_len].to_string()), &after[digit_len..])
        }
        None => (None, combined),
    };

    let parts: Vec<&str> = rest.split('_').collect();
    let is_name_part = |i: usize, part: &str| i == 0 || part.starts_with(|c: char| c.is_ascii_uppercase()) || part == "s" || part == "l";
    let name_len = parts.iter().enumerate().take_while(|(i, p)| is_name_part(*i, p)).count();
    let name = parts[..name_len].join("_");
    let carried = parts[name_len..].join("_");
    let carrying = (!carried.is_empty()).then_some(carried);

    // The stationary "F" variants use the regular teki's assets.
    let (name, stationary) = match name.as_str() {
        "FminiHoudai" => ("MiniHoudai".to_string(), true),
        "Fkabuto" => ("Kabuto".to_string(), true),
        _ => (name, false),
    };
    Identifier { spawn_method, name, carrying, stationary }
}
