//! The generation parameters for a sublevel ("CaveInfo") and the map units it can use.
//!
//! Types and rotation rules are ported from Caveripper's `caveinfo` module (MIT,
//! https://github.com/mayabyte/caveripper). File format reference:
//! https://pikmintkb.com/wiki/Cave_generation_parameters

use std::f32::consts::PI;

use crate::{data::CaveConfig, point::Point};

/// One floor of a cave: its FloorInfo plus the TekiInfo, ItemInfo, GateInfo and CapInfo
/// sections that follow it.
#[derive(Debug, Clone)]
pub struct CaveInfo {
    pub cave_cfg: CaveConfig,
    /// 0-indexed.
    pub floor_num: u32,
    pub max_main_objects: u32,
    pub max_treasures: u32,
    pub max_gates: u32,
    /// Excludes corridors and caps.
    pub num_rooms: u32,
    pub corridor_probability: f32,
    pub cap_probability: f32,
    pub has_geyser: bool,
    pub exit_plugged: bool,
    /// Every map unit in all 4 rotations, in the game's sorted order.
    pub cave_units: Vec<CaveUnit>,
    pub teki_info: Vec<TekiInfo>,
    pub item_info: Vec<ItemInfo>,
    pub gate_info: Vec<GateInfo>,
    pub cap_info: Vec<CapInfo>,
    pub is_final_floor: bool,
    /// Seconds until the Waterwraith falls; 0 = never.
    pub waterwraith_timer: f32,
}

impl CaveInfo {
    /// All teki in one spawn group.
    pub fn teki_group(&self, group: u32) -> impl Iterator<Item = &TekiInfo> {
        self.teki_info.iter().filter(move |teki| teki.group == group)
    }

    /// The most doors any single map unit on this floor has.
    pub fn max_num_doors_single_unit(&self) -> usize {
        self.cave_units.iter().map(|unit| unit.num_doors).max().unwrap_or_default()
    }

    /// The short sublevel name, e.g. "SCx6" or "CH12-CM2".
    pub fn name(&self) -> String {
        let names = &self.cave_cfg.shortened_names;
        if self.cave_cfg.is_challenge_mode {
            let other = names.iter().find(|n| !n.starts_with("CH")).cloned().unwrap_or_default();
            format!("{}-{}{}", names[0], other, self.floor_num + 1)
        } else {
            format!("{}{}", names[0], self.floor_num + 1)
        }
    }

    /// The long sublevel name, e.g. "Subterranean Complex 3".
    pub fn long_name(&self) -> String {
        format!("{} {}", self.cave_cfg.full_name, self.floor_num + 1)
    }

    pub fn is_challenge_mode(&self) -> bool {
        self.cave_cfg.is_challenge_mode
    }
}

/// An enemy, hazard, plant or other object spawned from TekiInfo.
#[derive(Debug, Clone, PartialEq)]
pub struct TekiInfo {
    pub game: String,
    pub internal_name: String,
    /// The treasure this teki carries, if any.
    pub carrying: Option<String>,
    pub minimum_amount: u32,
    pub filler_distribution_weight: u32,
    /// The spawn group ("type"): 0 easy, 1 hard, 5 seam, 6 plant, 8 special.
    pub group: u32,
    pub spawn_method: Option<String>,
    /// True for the stationary "F" variants (Fkabuto, FminiHoudai), whose names are
    /// normalized to the regular teki's name.
    pub stationary: bool,
}

/// A loose treasure.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemInfo {
    pub game: String,
    pub internal_name: String,
    pub min_amount: u8,
    pub filler_distribution_weight: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GateInfo {
    pub health: f32,
    pub spawn_distribution_weight: u32,
}

/// Objects that spawn in alcoves. Like [`TekiInfo`], except `group` controls how many
/// spawn rather than where.
#[derive(Debug, Clone, PartialEq)]
pub struct CapInfo {
    pub game: String,
    pub internal_name: String,
    pub carrying: Option<String>,
    pub minimum_amount: u32,
    pub filler_distribution_weight: u32,
    pub group: u8,
    pub spawn_method: Option<String>,
    pub stationary: bool,
}

impl CapInfo {
    /// Candypop Buds ("pom") get special treatment for falling cap teki and gates.
    pub fn is_candypop(&self) -> bool {
        self.internal_name.to_lowercase().contains("pom")
    }

    /// Every spawn method except "none" makes the object fall.
    pub fn is_falling(&self) -> bool {
        self.spawn_method.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomType {
    Room,
    Hallway,
    DeadEnd,
}

impl RoomType {
    /// Parses "room", "cap"/"alcove" or "hall"/"hallway" (any case).
    pub fn parse(name: &str) -> Option<RoomType> {
        match name.to_ascii_lowercase().as_str() {
            "room" => Some(RoomType::Room),
            "cap" | "alcove" => Some(RoomType::DeadEnd),
            "hall" | "hallway" => Some(RoomType::Hallway),
            _ => None,
        }
    }

    pub fn from_code(code: usize) -> Option<RoomType> {
        match code {
            0 => Some(RoomType::DeadEnd),
            1 => Some(RoomType::Room),
            2 => Some(RoomType::Hallway),
            _ => None,
        }
    }
}

impl std::fmt::Display for RoomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RoomType::Room => "room",
            RoomType::Hallway => "hallway",
            RoomType::DeadEnd => "cap",
        })
    }
}

/// One map unit in one rotation.
#[derive(Debug, Clone)]
pub struct CaveUnit {
    pub game: String,
    pub unit_folder_name: String,
    /// Identifies units with the same folder name (unit files can list a unit twice).
    pub name_id: u16,
    /// In grid cells.
    pub width: u16,
    pub height: u16,
    pub room_type: RoomType,
    pub num_doors: usize,
    pub doors: Vec<DoorUnit>,
    pub rotation: u16,
    pub spawnpoints: Vec<SpawnPoint>,
    pub waterboxes: Vec<Waterbox>,
    /// Points that guide carried objects, relative to the unit's center.
    pub waypoints: Vec<Waypoint>,
}

impl CaveUnit {
    /// The game sorts units by area, then by number of doors.
    pub(crate) fn sort_key(&self) -> (u16, usize) {
        (self.width * self.height, self.num_doors)
    }

    /// A copy of this unit turned by `rotation` quarter turns.
    pub fn copy_and_rotate_to(&self, rotation: u16) -> Self {
        let mut new_unit = self.clone();
        new_unit.rotation = (new_unit.rotation + rotation) % 4;
        if rotation % 2 == 1 {
            new_unit.width = self.height;
            new_unit.height = self.width;
        }

        // Copied from CaveGen's MapUnit.rotate.
        for door in new_unit.doors.iter_mut() {
            match door.direction {
                0 | 2 if rotation == 2 || rotation == 3 => door.side_lateral_offset = self.width - 1 - door.side_lateral_offset,
                1 | 3 if rotation == 1 || rotation == 2 => door.side_lateral_offset = self.height - 1 - door.side_lateral_offset,
                _ => {}
            }
            door.direction = (door.direction + rotation) % 4;
        }

        // Exact quarter turns, as the game does. Rotating with sin/cos of a float angle leaves
        // errors like 425.00003 that change which of two equally close waypoints is picked.
        for wp in new_unit.waypoints.iter_mut() {
            let [x, y, z] = wp.pos.0;
            wp.pos = Point(match rotation % 4 {
                0 => [x, y, z],
                1 => [-z, y, x],
                2 => [-x, y, -z],
                _ => [z, y, -x],
            });
        }

        let (cx, cz) = (self.width as f32 / 2.0, self.height as f32 / 2.0);
        let angle = rotation as f32 * PI / 2.0;
        for wb in new_unit.waterboxes.iter_mut() {
            wb.p1 = rotate_about_xz(wb.p1, cx, cz, angle);
            wb.p2 = rotate_about_xz(wb.p2, cx, cz, angle);
            // Keep p1 the low corner and p2 the high corner.
            if wb.p1[0] > wb.p2[0] {
                (wb.p1[0], wb.p2[0]) = (wb.p2[0], wb.p1[0]);
            }
            if wb.p1[2] > wb.p2[2] {
                (wb.p1[2], wb.p2[2]) = (wb.p2[2], wb.p1[2]);
            }
        }

        new_unit
    }

    pub fn has_start_spawnpoint(&self) -> bool {
        self.spawnpoints.iter().any(|sp| sp.group == 7)
    }

    /// Center of a door in grid coordinates relative to the unit's corner.
    pub fn center_of_door(&self, door: &DoorUnit) -> (f32, f32) {
        let offset = door.side_lateral_offset as f32 + 0.5;
        match door.direction {
            0 => (offset, 0.0),
            1 => (self.width as f32, offset),
            2 => (offset, self.height as f32),
            _ => (0.0, offset),
        }
    }
}

/// Rotates counter-clockwise about (cx, cz) in the XZ plane, as Caveripper does for waterboxes.
fn rotate_about_xz(p: Point, cx: f32, cz: f32, angle: f32) -> Point {
    let (s, c) = (angle.sin(), angle.cos());
    let (dx, dz) = (p[0] - cx, p[2] - cz);
    Point([dx * c - dz * s + cx, p[1], dx * s + dz * c + cz])
}

/// A cuboid of water in a map unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Waterbox {
    pub p1: Point,
    pub p2: Point,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    pub pos: Point,
    pub r: f32,
    pub index: usize,
    pub links: Vec<usize>,
}

/// A door: an opening one grid cell wide where map units connect.
#[derive(Debug, Clone, PartialEq)]
pub struct DoorUnit {
    /// 0 = up (-z), 1 = right (+x), 2 = down (+z), 3 = left (-x).
    pub direction: u16,
    pub side_lateral_offset: u16,
    pub waypoint_index: usize,
    pub num_links: usize,
    /// Straight-line links to the unit's other doors, used for door scores.
    pub door_links: Vec<DoorLink>,
}

impl DoorUnit {
    pub fn facing(&self, other: &DoorUnit) -> bool {
        (self.direction as i32 - other.direction as i32).abs() == 2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoorLink {
    pub distance: f32,
    pub door_id: usize,
    pub tekiflag: bool,
}

/// Where objects can spawn in a map unit, relative to the unit's center.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnPoint {
    /// 0 easy, 1 hard, 2 treasure, 4 hole/geyser, 6 plant, 7 ship/onion, 8 special,
    /// 9 alcove/hallway spot (added by the parser; not in the game files).
    pub group: u16,
    pub pos: Point,
    pub angle_degrees: f32,
    pub radius: f32,
    pub min_num: u16,
    pub max_num: u16,
}
