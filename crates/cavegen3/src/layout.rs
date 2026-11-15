//! A generated sublevel layout.
//!
//! Everything is stored in flat arrays and referenced by index: units, their doors, and
//! their spawn points. This keeps generation free of reference counting and per-candidate
//! allocation.

use std::sync::OnceLock;

use crate::{
    caveinfo::{CapInfo, CaveInfo, CaveUnit, DoorUnit, GateInfo, ItemInfo, SpawnPoint, TekiInfo},
    generate,
    point::Point,
    waypoint::WaypointGraph,
};

/// Something that spawns in a layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpawnObject<'a> {
    /// A teki and its offset from the spawn point (only group 0 teki are offset).
    Teki(&'a TekiInfo, Point),
    /// An alcove teki and how many of it spawned.
    CapTeki(&'a CapInfo, u32),
    Item(&'a ItemInfo),
    /// A gate and the direction of the door it blocks.
    Gate(&'a GateInfo, u16),
    /// The exit hole; true if it's plugged.
    Hole(bool),
    /// The geyser; true if it's the challenge mode exit geyser.
    Geyser(bool),
    Ship,
    /// A Colossal Caverns onion and its color index.
    Onion(u32),
}

/// A map unit placed in the layout.
#[derive(Debug, Clone)]
pub struct PlacedUnit<'a> {
    pub unit: &'a CaveUnit,
    /// Grid position of the unit's corner.
    pub x: i32,
    pub z: i32,
    /// Index of this unit's first door in [`Layout::doors`]; it has `unit.doors.len()` doors.
    pub first_door: u32,
    /// Index of this unit's first spawn point in [`Layout::spawnpoints`].
    pub first_spawnpoint: u32,
    pub teki_score: i32,
    /// The game's unit score, or -1 if scoring never reached this unit.
    pub total_score: i32,
}

impl PlacedUnit<'_> {
    pub fn door_indices(&self) -> std::ops::Range<usize> {
        self.first_door as usize..self.first_door as usize + self.unit.doors.len()
    }

    pub fn spawnpoint_indices(&self) -> std::ops::Range<usize> {
        self.first_spawnpoint as usize..self.first_spawnpoint as usize + self.unit.spawnpoints.len()
    }
}

/// A door of a placed unit.
#[derive(Debug, Clone)]
pub struct PlacedDoor<'a> {
    pub door: &'a DoorUnit,
    /// Grid position.
    pub x: i32,
    pub z: i32,
    /// Index of the unit this door belongs to.
    pub unit: u32,
    /// Index of the door this one connects to.
    pub adjacent: Option<u32>,
    pub marked_as_cap: bool,
    /// The game's door score, or -1 if scoring never reached this door.
    pub door_score: i32,
    pub seam_teki_score: i32,
    /// A seam teki or gate on this door. Both doors of a seam hold the same object.
    pub seam: Option<SpawnObject<'a>>,
}

impl PlacedDoor<'_> {
    /// Whether two doors face each other at the same spot.
    pub fn lines_up_with(&self, other: &PlacedDoor) -> bool {
        self.x == other.x && self.z == other.z && self.door.facing(other.door)
    }

    /// The center of the door in game coordinates, where seam objects spawn.
    pub fn center(&self) -> Point {
        let mut pos = Point([self.x as f32 * 170.0, 0.0, self.z as f32 * 170.0]);
        if self.door.direction % 2 == 0 {
            pos[0] += 85.0;
        } else {
            pos[2] += 85.0;
        }
        pos
    }
}

/// A spawn point of a placed unit, in game coordinates.
#[derive(Debug, Clone)]
pub struct PlacedSpawnPoint<'a> {
    pub spawnpoint: &'a SpawnPoint,
    pub pos: Point,
    pub angle: f32,
    /// Index of the unit this spawn point belongs to.
    pub unit: u32,
    pub hole_score: i32,
    pub treasure_score: i32,
    pub contains: Vec<SpawnObject<'a>>,
}

impl PlacedSpawnPoint<'_> {
    pub fn group(&self) -> u16 {
        self.spawnpoint.group
    }
}

/// A fully generated layout.
#[derive(Debug, Clone)]
pub struct Layout<'a> {
    pub caveinfo: &'a CaveInfo,
    pub seed: u32,
    pub units: Vec<PlacedUnit<'a>>,
    pub doors: Vec<PlacedDoor<'a>>,
    pub spawnpoints: Vec<PlacedSpawnPoint<'a>>,
    /// Spawn point indices of the ship, exit hole and geyser.
    pub ship: Option<u32>,
    pub hole: Option<u32>,
    pub geyser: Option<u32>,
    /// Every object in the order it was placed (the ship, exits, teki, treasures, alcove teki,
    /// gates, onions), as CaveGen's -drawSpawnOrder numbers them.
    pub spawn_order: Vec<Placement>,
    /// Built on first use by [`Layout::waypoint_graph`].
    pub(crate) waypoints: OnceLock<WaypointGraph>,
}

/// Where a placed object is: an entry of a spawn point's `contains`, or a door seam (the lower
/// of the two door indices, which both hold it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    SpawnPoint { index: u32, slot: u32 },
    Seam { door: u32 },
}

impl<'a> Layout<'a> {
    /// The carry-path graph, built the first time it's needed.
    pub fn waypoint_graph(&self) -> &WaypointGraph {
        self.waypoints.get_or_init(|| WaypointGraph::build(self))
    }

    /// Generates the layout for `seed`.
    pub fn generate(seed: u32, caveinfo: &'a CaveInfo) -> Layout<'a> {
        generate::generate(seed, caveinfo)
    }

    /// The doors of one unit.
    pub fn unit_doors(&self, unit: usize) -> &[PlacedDoor<'a>] {
        &self.doors[self.units[unit].door_indices()]
    }

    /// The spawn points of one unit.
    pub fn unit_spawnpoints(&self, unit: usize) -> &[PlacedSpawnPoint<'a>] {
        &self.spawnpoints[self.units[unit].spawnpoint_indices()]
    }

    /// Every spawned object with its position: first everything in spawn points (by unit,
    /// then spawn point), then seam objects, each once.
    pub fn spawn_objects(&self) -> impl Iterator<Item = (&SpawnObject<'a>, Point)> + '_ {
        let in_spawnpoints = self.spawnpoints.iter().flat_map(|sp| {
            sp.contains.iter().map(move |so| match so {
                SpawnObject::Teki(_, offset) => (so, sp.pos + *offset),
                _ => (so, sp.pos),
            })
        });
        let in_seams = self.doors.iter().enumerate().filter_map(|(i, door)| {
            // Both doors of a seam hold the object; report it at the first one.
            let first = door.adjacent.is_none_or(|adj| i < adj as usize);
            door.seam.as_ref().filter(|_| first).map(|so| (so, door.center()))
        });
        in_spawnpoints.chain(in_seams)
    }
}
