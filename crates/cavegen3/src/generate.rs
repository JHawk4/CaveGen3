//! Cave generation.
//!
//! This follows Caveripper's generator (MIT, https://github.com/mayabyte/caveripper), which
//! follows JHawk's CaveGen and the game (pikmin2 decomp, `plugProjectNishimuraU`), call for
//! call: every RNG call and float operation happens in the same order, so layouts match
//! exactly. The data structures are new: units, doors and spawn points live in flat arrays,
//! the open doors are tracked in a sorted list, and spawn points are only created once map
//! placement is done.

use std::cmp::{max, min};

use crate::{
    caveinfo::{CapInfo, CaveInfo, CaveUnit, ItemInfo, RoomType, TekiInfo},
    layout::{Layout, PlacedDoor, PlacedSpawnPoint, PlacedUnit, SpawnObject},
    math,
    point::Point,
    rng::Rng,
};

type UnitId = u16;

pub(crate) fn generate(seed: u32, caveinfo: &CaveInfo) -> Layout<'_> {
    let mut g = Generator {
        ci: caveinfo,
        rng: Rng::new(seed),
        seed,
        units: Vec::new(),
        doors: Vec::new(),
        open: Vec::new(),
        cap_queue: Vec::new(),
        room_queue: Vec::new(),
        corridor_queue: Vec::new(),
        rooms_placed: 0,
        slots: [0; 10],
        weight_sums: [0; 10],
        min_teki_0: 0,
        map_min_x: 0,
        map_min_z: 0,
        map_max_x: 0,
        map_max_z: 0,
        diameter_36: false,
        marked_caps: false,
        placed_teki: 0,
        spawnpoints: Vec::new(),
        ship: None,
        hole: None,
        geyser: None,
    };
    g.run();
    Layout {
        caveinfo,
        seed,
        units: g.units,
        doors: g.doors,
        spawnpoints: g.spawnpoints,
        ship: g.ship,
        hole: g.hole,
        geyser: g.geyser,
    }
}

struct Generator<'a> {
    ci: &'a CaveInfo,
    rng: Rng,
    seed: u32,
    units: Vec<PlacedUnit<'a>>,
    doors: Vec<PlacedDoor<'a>>,
    /// Indices of doors with nothing attached, in ascending order (the game's iteration order).
    open: Vec<u32>,
    cap_queue: Vec<UnitId>,
    room_queue: Vec<UnitId>,
    corridor_queue: Vec<UnitId>,
    rooms_placed: u32,
    /// Teki slots allocated to each spawn group.
    slots: [u32; 10],
    weight_sums: [u32; 10],
    min_teki_0: u32,
    map_min_x: i32,
    map_min_z: i32,
    map_max_x: i32,
    map_max_z: i32,
    diameter_36: bool,
    marked_caps: bool,
    placed_teki: u32,
    spawnpoints: Vec<PlacedSpawnPoint<'a>>,
    ship: Option<u32>,
    hole: Option<u32>,
    geyser: Option<u32>,
}

/// A unit position that passed the placement checks.
#[derive(Clone, Copy)]
struct Candidate {
    unit: UnitId,
    x: i32,
    z: i32,
}

impl<'a> Generator<'a> {
    fn unit(&self, id: UnitId) -> &'a CaveUnit {
        &self.ci.cave_units[id as usize]
    }

    fn run(&mut self) {
        self.sort_units_and_allocate_teki();
        self.place_map_units();
        self.create_spawnpoints_and_recenter();
        self.place_ship();
        self.set_score();
        if !self.ci.is_final_floor {
            self.place_hole(SpawnObject::Hole(self.ci.exit_plugged));
        }
        if self.ci.is_final_floor || self.ci.has_geyser {
            self.place_hole(SpawnObject::Geyser(self.ci.is_challenge_mode() && self.ci.is_final_floor));
        }
        self.place_seam_teki();
        self.place_special_teki();
        self.place_hard_teki();
        self.place_easy_teki();
        self.set_score();
        self.place_plants();
        self.place_items();
        self.place_cap_teki();
        self.place_gates();
        if self.ci.cave_cfg.is_colossal_caverns() {
            self.place_onions();
        }
    }

    // ---------------------------------------------------------------------------------
    // Setup

    fn sort_units_and_allocate_teki(&mut self) {
        // mapUnitsInitialSorting
        for (i, unit) in self.ci.cave_units.iter().enumerate() {
            match unit.room_type {
                RoomType::DeadEnd => self.cap_queue.push(i as UnitId),
                RoomType::Room => self.room_queue.push(i as UnitId),
                RoomType::Hallway => self.corridor_queue.push(i as UnitId),
            }
        }
        self.rng.rand_backs(&mut self.cap_queue);
        self.rng.rand_backs(&mut self.room_queue);
        self.rng.rand_backs(&mut self.corridor_queue);

        // allocateEnemySlots
        let mut used_for_min = 0;
        for group in [0, 1, 5, 8] {
            for teki in self.ci.teki_group(group) {
                self.slots[group as usize] += teki.minimum_amount;
                self.weight_sums[group as usize] += teki.filler_distribution_weight;
                used_for_min += teki.minimum_amount;
            }
        }
        self.min_teki_0 = self.slots[0];
        for _ in 0..self.ci.max_main_objects.saturating_sub(used_for_min) {
            if let Some(group) = self.rng.rand_index_weight(&self.weight_sums) {
                self.slots[group] += 1;
            }
        }
    }

    // ---------------------------------------------------------------------------------
    // Map unit placement

    fn place_map_units(&mut self) {
        let start = *self
            .room_queue
            .iter()
            .find(|&&id| self.unit(id).has_start_spawnpoint())
            .expect("no room with a start spawn point");
        self.place_unit(Candidate { unit: start, x: 0, z: 0 });

        if self.open.is_empty() {
            return;
        }
        // The game gives up after 500 placement steps (RandMapUnit::setMapUnit).
        for _ in 0..500 {
            let mut placed = if self.rooms_placed < self.ci.num_rooms { self.try_place_room() } else { self.try_place_hallway() };
            if placed.is_none() {
                placed = self.try_place_cap();
            }
            if let Some(candidate) = placed {
                self.place_unit(candidate);
            }

            if !self.open.is_empty() {
                continue;
            }
            self.change_cap_to_hallway();
            if !self.open.is_empty() {
                continue;
            }
            // With no open doors left, the game merges hallways and stops.
            self.change_two_hallways_to_one();
            break;
        }
        assert!(
            self.open.is_empty(),
            "Map units still had open doors after the game's 500 placement steps. The game doesn't handle this case. Seed: {:#X}, Sublevel: {}",
            self.seed,
            self.ci.name()
        );
    }

    /// Places a room, hallway or cap on a random open door, preferring rooms.
    fn try_place_room(&mut self) -> Option<Candidate> {
        let destination = self.open[self.rng.rand_int(self.open.len() as u32) as usize];

        let mut corridor_probability = self.ci.corridor_probability;
        if self.diameter_36 {
            corridor_probability = 0.0;
        }
        let dest_unit = self.doors[destination as usize].unit as usize;
        if self.units[dest_unit].unit.room_type == RoomType::Room {
            corridor_probability *= 2.0;
        }
        let priority = if self.rng.rand_f32() < corridor_probability {
            [RoomType::Hallway, RoomType::Room, RoomType::DeadEnd]
        } else {
            [RoomType::Room, RoomType::Hallway, RoomType::DeadEnd]
        };

        for room_type in priority {
            let queue = match room_type {
                RoomType::Room => self.room_queue.clone(),
                RoomType::DeadEnd => self.cap_queue.clone(),
                RoomType::Hallway => {
                    self.shuffle_corridor_priority();
                    self.corridor_queue.clone()
                }
            };
            for id in queue {
                let num_doors = self.unit(id).num_doors;
                let mut door_priority: Vec<usize> = (0..num_doors).collect();
                self.rng.rand_swaps(&mut door_priority);
                for door_index in door_priority {
                    if let Some(c) = self.try_place_unit_at(destination, id, door_index) {
                        return Some(c);
                    }
                }
            }
        }
        None
    }

    /// Once all rooms are placed, grows hallways from open doors towards other open doors.
    fn try_place_hallway(&mut self) -> Option<Candidate> {
        self.mark_random_open_doors_as_caps();

        let mut hallway_queue: Vec<UnitId> = self
            .corridor_queue
            .iter()
            .copied()
            .filter(|&id| {
                let u = self.unit(id);
                u.width == 1 && u.height == 1 && u.num_doors == 2
            })
            .collect();
        self.rng.rand_swaps(&mut hallway_queue);

        for &open_door in self.open.clone().iter() {
            let od = &self.doors[open_door as usize];
            if od.marked_as_cap {
                continue;
            }

            // The closest open door on another unit in the 10x10 area in front of this one.
            let mut link_door = None;
            let mut link_dist = i32::MAX;
            for &candidate in self.open.iter() {
                let cd = &self.doors[candidate as usize];
                if od.unit == cd.unit {
                    continue;
                }
                let dx = cd.x - od.x;
                let dz = cd.z - od.z;
                if dx.abs() >= 10 || dz.abs() >= 10 {
                    continue;
                }
                let skip = match od.door.direction {
                    0 => dz > 0,
                    1 => dx < 0,
                    2 => dz < 0,
                    3 => dx > 0,
                    _ => false,
                };
                if skip {
                    continue;
                }
                let distance = dx.abs() + dz.abs();
                if distance < link_dist {
                    link_door = Some(candidate);
                    link_dist = distance;
                }
            }
            let Some(link_door) = link_door else { continue };

            let ld = &self.doors[link_door as usize];
            let dx = ld.x - od.x;
            let dz = ld.z - od.z;
            let link_door_dir = ld.door.direction;
            let open_door_dir = od.door.direction;

            // Which way the hallway should turn ("snaking" towards the link door). Copied
            // verbatim from Caveripper.
            let priority = match open_door_dir {
                0 => {
                    if dz > -2 {
                        if dx >= 0 {
                            1
                        } else {
                            3
                        }
                    } else {
                        match dx {
                            _ if dx < -1 => 3,
                            -1 => {
                                if link_door_dir == 2 || link_door_dir == 3 {
                                    3
                                } else {
                                    0
                                }
                            }
                            0 => {
                                if link_door_dir == 0 || link_door_dir == 3 {
                                    3
                                } else {
                                    0
                                }
                            }
                            1 => {
                                if link_door_dir == 1 || link_door_dir == 2 {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ if dx > 1 => 1,
                            _ => unreachable!(),
                        }
                    }
                }
                1 => {
                    if dx == 0 {
                        if dz > 0 {
                            2
                        } else {
                            0
                        }
                    } else {
                        match dz {
                            _ if dz < -1 => 0,
                            -1 => {
                                if link_door_dir == 0 || link_door_dir == 3 {
                                    0
                                } else {
                                    1
                                }
                            }
                            0 => {
                                if link_door_dir == 0 || link_door_dir == 1 {
                                    0
                                } else {
                                    1
                                }
                            }
                            1 => {
                                if link_door_dir == 2 || link_door_dir == 3 {
                                    2
                                } else {
                                    1
                                }
                            }
                            _ if dz > 1 => 2,
                            _ => unreachable!(),
                        }
                    }
                }
                2 => {
                    if dz == 0 {
                        if dx > 0 {
                            1
                        } else {
                            3
                        }
                    } else {
                        match dx {
                            _ if dx < -1 => 3,
                            -1 => {
                                if link_door_dir == 0 || link_door_dir == 3 {
                                    3
                                } else {
                                    2
                                }
                            }
                            0 => {
                                if link_door_dir == 2 || link_door_dir == 3 {
                                    3
                                } else {
                                    2
                                }
                            }
                            1 => {
                                if link_door_dir == 0 || link_door_dir == 1 {
                                    1
                                } else {
                                    2
                                }
                            }
                            _ if dx > 1 => 1,
                            _ => unreachable!(),
                        }
                    }
                }
                3 => {
                    if dx > -2 {
                        if dz > 0 {
                            2
                        } else {
                            0
                        }
                    } else {
                        match dz {
                            _ if dz < -1 => 0,
                            -1 => {
                                if link_door_dir == 0 || link_door_dir == 1 {
                                    0
                                } else {
                                    3
                                }
                            }
                            0 => {
                                if link_door_dir == 0 || link_door_dir == 3 {
                                    0
                                } else {
                                    3
                                }
                            }
                            1 => {
                                if link_door_dir == 1 || link_door_dir == 2 {
                                    2
                                } else {
                                    3
                                }
                            }
                            _ if dz > 1 => 2,
                            _ => unreachable!(),
                        }
                    }
                }
                _ => panic!("Invalid direction in hallway snaking"),
            };

            // Try a hallway with the desired turn, then a straight one.
            let dir_hallway_0 = (open_door_dir + 2) % 4;
            for dir_hallway_1 in [priority, open_door_dir] {
                for &id in hallway_queue.iter() {
                    let u = self.unit(id);
                    let (d0, d1) = (u.doors[0].direction, u.doors[1].direction);
                    let placed = if d0 == dir_hallway_0 && d1 == dir_hallway_1 {
                        self.try_place_unit_at(open_door, id, 0)
                    } else if d0 == dir_hallway_1 && d1 == dir_hallway_0 {
                        self.try_place_unit_at(open_door, id, 1)
                    } else {
                        None
                    };
                    if placed.is_some() {
                        return placed;
                    }
                }
            }
        }
        None
    }

    /// Closes off an open door with whatever fits, trying caps first.
    fn try_place_cap(&mut self) -> Option<Candidate> {
        let max_doors = self.ci.max_num_doors_single_unit();
        for &open_door in self.open.clone().iter() {
            for room_type in [RoomType::DeadEnd, RoomType::Hallway, RoomType::Room] {
                let queue = match room_type {
                    RoomType::Room => self.room_queue.clone(),
                    RoomType::DeadEnd => self.cap_queue.clone(),
                    RoomType::Hallway => self.corridor_queue.clone(),
                };
                for num_doors in 1..=max_doors {
                    for &id in queue.iter() {
                        if self.unit(id).num_doors != num_doors {
                            continue;
                        }
                        let mut door_priority: Vec<usize> = (0..num_doors).collect();
                        self.rng.rand_swaps(&mut door_priority);
                        for door_index in door_priority {
                            if let Some(c) = self.try_place_unit_at(open_door, id, door_index) {
                                return Some(c);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Checks whether unit `id`, attached by its door `door_index` to `destination`, fits.
    fn try_place_unit_at(&self, destination: u32, id: UnitId, door_index: usize) -> Option<Candidate> {
        let dest = &self.doors[destination as usize];
        let unit = self.unit(id);
        let new_door = &unit.doors[door_index];
        if !dest.door.facing(new_door) {
            return None;
        }
        let offset = new_door.side_lateral_offset as i32;
        let (cx, cz) = match new_door.direction {
            0 => (dest.x - offset, dest.z),
            1 => (dest.x - unit.width as i32, dest.z - offset),
            2 => (dest.x - offset, dest.z - unit.height as i32),
            _ => (dest.x, dest.z - offset),
        };

        // No overlap with placed units.
        if self.units.iter().any(|p| boxes_overlap(p.x, p.z, p.unit.width, p.unit.height, cx, cz, unit.width, unit.height)) {
            return None;
        }

        let lines_up = |a: (i32, i32, u16), b: (i32, i32, u16)| a.0 == b.0 && a.1 == b.1 && (a.2 as i32 - b.2 as i32).abs() == 2;
        let space_in_front = |(x, z, dir): (i32, i32, u16)| (x - i32::from(dir == 3), z - i32::from(dir == 0));
        let open = || {
            self.open.iter().map(|&d| {
                let d = &self.doors[d as usize];
                (d.x, d.z, d.door.direction)
            })
        };
        let new_doors = || {
            unit.doors.iter().map(|d| {
                let (x, z) = door_position(unit, d, cx, cz);
                (x, z, d.direction)
            })
        };

        // Each new door must meet an open door or face empty space, not a wall.
        for nd in new_doors() {
            if open().any(|od| lines_up(nd, od)) {
                continue;
            }
            let (sx, sz) = space_in_front(nd);
            if self.units.iter().any(|p| boxes_overlap(sx, sz, 1, 1, p.x, p.z, p.unit.width, p.unit.height)) {
                return None;
            }
        }
        // And each open door must meet a new door or not face into the new unit.
        for od in open() {
            if new_doors().any(|nd| lines_up(od, nd)) {
                continue;
            }
            let (sx, sz) = space_in_front(od);
            if boxes_overlap(sx, sz, 1, 1, cx, cz, unit.width, unit.height) {
                return None;
            }
        }
        Some(Candidate { unit: id, x: cx, z: cz })
    }

    /// Adds a unit, attaches its doors, reshuffles the queues and updates the map size.
    fn place_unit(&mut self, c: Candidate) {
        let unit = self.unit(c.unit);
        let unit_idx = self.units.len() as u32;
        let first_door = self.doors.len() as u32;
        for door in unit.doors.iter() {
            let (x, z) = door_position(unit, door, c.x, c.z);
            self.doors.push(PlacedDoor {
                door,
                x,
                z,
                unit: unit_idx,
                adjacent: None,
                marked_as_cap: false,
                door_score: 0,
                seam_teki_score: 0,
                seam: None,
            });
        }
        self.units.push(PlacedUnit { unit, x: c.x, z: c.z, first_door, first_spawnpoint: 0, teki_score: 0, total_score: 0 });
        if unit.room_type == RoomType::Room {
            self.rooms_placed += 1;
        }

        // Attach doors that line up with open doors. The game checks each new door against
        // the doors open at that moment, in order.
        let candidates = self.open.clone();
        for nd in first_door..self.doors.len() as u32 {
            for &od in candidates.iter() {
                if self.doors[od as usize].adjacent.is_none() && self.doors[nd as usize].lines_up_with(&self.doors[od as usize]) {
                    self.doors[nd as usize].adjacent = Some(od);
                    self.doors[od as usize].adjacent = Some(nd);
                }
            }
        }
        self.open.retain(|&d| self.doors[d as usize].adjacent.is_none());
        self.open.extend((first_door..self.doors.len() as u32).filter(|&d| self.doors[d as usize].adjacent.is_none()));

        self.shuffle_unit_priority(unit);

        self.map_min_x = min(self.map_min_x, c.x);
        self.map_min_z = min(self.map_min_z, c.z);
        self.map_max_x = max(self.map_max_x, c.x + unit.width as i32);
        self.map_max_z = max(self.map_max_z, c.z + unit.height as i32);
        self.diameter_36 = self.map_max_x - self.map_min_x >= 36 || self.map_max_z - self.map_min_z >= 36;
    }

    /// Removes a unit whose doors have already been detached from their neighbors.
    fn remove_unit(&mut self, idx: usize) {
        let removed = self.units.remove(idx);
        let start = removed.first_door;
        let count = removed.unit.doors.len() as u32;
        self.doors.drain(start as usize..(start + count) as usize);
        let shift = |d: u32| if d >= start + count { Some(d - count) } else if d >= start { None } else { Some(d) };
        for unit in self.units.iter_mut() {
            if unit.first_door > start {
                unit.first_door -= count;
            }
        }
        for door in self.doors.iter_mut() {
            door.adjacent = door.adjacent.and_then(shift);
            if door.unit as usize > idx {
                door.unit -= 1;
            }
        }
        self.rebuild_open_list();
    }

    fn rebuild_open_list(&mut self) {
        self.open.clear();
        self.open.extend((0..self.doors.len() as u32).filter(|&d| self.doors[d as usize].adjacent.is_none()));
    }

    /// Clears the other side's link for every door of a unit.
    fn detach_neighbors(&mut self, unit_idx: usize) {
        for d in self.units[unit_idx].door_indices() {
            if let Some(adj) = self.doors[d].adjacent {
                self.doors[adj as usize].adjacent = None;
            }
        }
    }

    /// After a unit is placed, the game reshuffles the queue it came from so less-used units
    /// come first.
    fn shuffle_unit_priority(&mut self, placed: &CaveUnit) {
        match placed.room_type {
            RoomType::DeadEnd => self.rng.rand_backs(&mut self.cap_queue),
            RoomType::Hallway => self.rng.rand_backs(&mut self.corridor_queue),
            RoomType::Room => {
                // Count placed rooms by name, in order of first appearance.
                let mut counts: Vec<(u16, usize)> = Vec::new();
                for u in self.units.iter().filter(|u| u.unit.room_type == RoomType::Room) {
                    match counts.iter_mut().find(|(name, _)| *name == u.unit.name_id) {
                        Some(entry) => entry.1 += 1,
                        None => counts.push((u.unit.name_id, 1)),
                    }
                }
                // The game's swapping sort, ascending by count.
                for i in 0..counts.len() {
                    for j in i + 1..counts.len() {
                        if counts[i].1 > counts[j].1 {
                            counts.swap(i, j);
                        }
                    }
                }
                // Move each name's entries to the back, least-used names first.
                for (name, _) in counts {
                    let mut matching = Vec::new();
                    let mut i = 0;
                    while i < self.room_queue.len() {
                        if self.unit(self.room_queue[i]).name_id == name {
                            matching.push(self.room_queue.remove(i));
                        } else {
                            i += 1;
                        }
                    }
                    self.rng.rand_backs_n(&mut matching, 4);
                    self.room_queue.append(&mut matching);
                }
            }
        }
    }

    /// Orders the corridor queue by number of doors, depending on how many doors are open.
    fn shuffle_corridor_priority(&mut self) {
        let max_doors = self.ci.max_num_doors_single_unit();
        let num_open = self.open.len();
        let mut priority: Vec<usize> = if num_open < 4 {
            (0..max_doors).map(|i| max_doors - i).collect()
        } else {
            (1..=max_doors).collect()
        };
        if (4..10).contains(&num_open) {
            self.rng.rand_swaps(&mut priority);
        }
        let mut sorted = Vec::with_capacity(self.corridor_queue.len());
        for num_doors in priority {
            let mut i = 0;
            while i < self.corridor_queue.len() {
                if self.unit(self.corridor_queue[i]).num_doors == num_doors {
                    sorted.push(self.corridor_queue.remove(i));
                } else {
                    i += 1;
                }
            }
        }
        self.corridor_queue = sorted;
    }

    /// Once all rooms are placed, marks up to 16 random open doors as caps, so hallways don't
    /// grow from them. Happens once per layout.
    fn mark_random_open_doors_as_caps(&mut self) {
        if self.marked_caps {
            return;
        }
        self.marked_caps = true;
        let mut marked = 0;
        for &d in self.open.iter() {
            if self.rng.rand_f32() < self.ci.cap_probability {
                self.doors[d as usize].marked_as_cap = true;
                marked += 1;
                if marked >= 16 {
                    break;
                }
            }
        }
    }

    /// Hallway 1x1 units running north-south (doors facing 0 and 2), by name, in queue order.
    fn straight_hallway_names(&self, height: u16) -> Vec<u16> {
        self.corridor_queue
            .iter()
            .map(|&id| self.unit(id))
            .filter(|u| u.room_type == RoomType::Hallway && u.width == 1 && u.height == height && u.num_doors == 2)
            .filter(|u| u.doors[0].direction == 0 && u.doors[1].direction == 2)
            .map(|u| u.name_id)
            .collect()
    }

    /// changeCapToHallMapUnit: replaces one alcove that has a hallway directly behind it with
    /// a hallway, and removes the hallway behind it.
    fn change_cap_to_hallway(&mut self) {
        let names = self.straight_hallway_names(1);
        if names.is_empty() {
            return;
        }
        for i in 0..self.units.len() {
            let cap = &self.units[i];
            if cap.unit.room_type != RoomType::DeadEnd {
                continue;
            }
            let cap_door = cap.first_door as usize;
            let cap_door_dir = self.doors[cap_door].door.direction;
            let (space_x, space_z) = match cap_door_dir {
                0 => (cap.x, cap.z + 1),
                1 => (cap.x - 1, cap.z),
                2 => (cap.x, cap.z - 1),
                _ => (cap.x + 1, cap.z),
            };
            let Some(behind) = self.units.iter().position(|u| u.unit.room_type == RoomType::Hallway && u.x == space_x && u.z == space_z)
            else {
                continue;
            };

            // Detach the cap's neighbor and the hallway's neighbors, keeping where the cap was attached.
            let attach_to = self.doors[cap_door].adjacent.expect("alcove door isn't attached") as usize;
            self.doors[attach_to].adjacent = None;
            let attach_pos = (self.doors[attach_to].x, self.doors[attach_to].z, self.doors[attach_to].door.direction);
            self.detach_neighbors(behind);

            // Remove the later one first so the other index stays valid.
            let (first, second) = if i > behind { (i, behind) } else { (behind, i) };
            self.remove_unit(first);
            self.remove_unit(second);
            // Door indices shift when units are removed; find the door again by position.
            let attach_to = self.find_open_door(attach_pos);

            let chosen = names[self.rng.rand_int(names.len() as u32) as usize];
            let corridor_queue = self.corridor_queue.clone();
            for id in corridor_queue {
                let u = self.unit(id);
                if u.name_id == chosen && u.doors[0].direction == cap_door_dir {
                    if let Some(c) = self.try_place_unit_at(attach_to, id, 0) {
                        self.place_unit(c);
                        return;
                    }
                }
            }
            panic!("Deleted cap in cap-to-hallway replacement step but couldn't replace it with a hallway!");
        }
    }

    /// changeTwoToOneMapUnit: merges pairs of adjacent 1x1 straight hallways into 2x1 ones.
    fn change_two_hallways_to_one(&mut self) {
        let names_1x1 = self.straight_hallway_names(1);
        let names_2x1 = self.straight_hallway_names(2);
        if names_1x1.is_empty() || names_2x1.is_empty() {
            return;
        }
        let mut num_units = self.units.len();
        let mut unit_1 = 0;
        while unit_1 < num_units {
            if !names_1x1.contains(&self.units[unit_1].unit.name_id) {
                unit_1 += 1;
                continue;
            }
            // Look for another 1x1 hallway on either side.
            let first_door = self.units[unit_1].first_door as usize;
            let mut found = None;
            let mut md = first_door + 1;
            for j in 0..2 {
                md = first_door + j;
                let od = self.doors[md].adjacent.expect("door isn't attached") as usize;
                let unit_2 = self.doors[od].unit as usize;
                if names_1x1.contains(&self.units[unit_2].unit.name_id) {
                    found = Some((od, unit_2));
                    break;
                }
            }
            let Some((od, unit_2)) = found else {
                unit_1 += 1;
                continue;
            };

            // Expand from the door beyond the far end of the pair.
            let (u1, u2) = (&self.units[unit_1], &self.units[unit_2]);
            let expand_from = if u1.x > u2.x || u1.z < u2.z {
                let far = u1.first_door as usize + self.doors[md].door.door_links[0].door_id;
                self.doors[far].adjacent.expect("door isn't attached")
            } else {
                let far = u2.first_door as usize + self.doors[od].door.door_links[0].door_id;
                self.doors[far].adjacent.expect("door isn't attached")
            };
            let desired_direction: u16 = if u1.x == u2.x { 0 } else { 1 };

            self.detach_neighbors(unit_1);
            self.detach_neighbors(unit_2);
            let expand_pos = {
                let d = &self.doors[expand_from as usize];
                (d.x, d.z, d.door.direction)
            };
            let (first, second) = if unit_1 > unit_2 { (unit_1, unit_2) } else { (unit_2, unit_1) };
            self.remove_unit(first);
            self.remove_unit(second);
            num_units -= 2;
            let expand_from = self.find_open_door(expand_pos);

            let chosen = names_2x1[self.rng.rand_int(names_2x1.len() as u32) as usize];
            let mut placed = false;
            let corridor_queue = self.corridor_queue.clone();
            for id in corridor_queue {
                let u = self.unit(id);
                if u.name_id == chosen && u.doors[0].direction == desired_direction {
                    if let Some(c) = self.try_place_unit_at(expand_from, id, 0) {
                        self.place_unit(c);
                        num_units += 1;
                        placed = true;
                        unit_1 = 0;
                        break;
                    }
                }
            }
            assert!(
                placed,
                "Deleted hallway units to combine but couldn't place a new hallway unit in their place! Seed: {:#X}, Sublevel: {}",
                self.seed,
                self.ci.name()
            );
        }
    }

    /// Finds an open door again after units were removed, by its position and direction.
    fn find_open_door(&self, (x, z, dir): (i32, i32, u16)) -> u32 {
        *self
            .open
            .iter()
            .find(|&&d| {
                let door = &self.doors[d as usize];
                door.x == x && door.z == z && door.door.direction == dir
            })
            .expect("door to expand from is no longer open")
    }

    // ---------------------------------------------------------------------------------
    // Spawn points, ship and scores

    /// Creates spawn points for every unit, then shifts the map so all coordinates are >= 0.
    /// Positions are computed from the unshifted coordinates and then shifted, in the same
    /// float operations as the game, so they round the same way.
    fn create_spawnpoints_and_recenter(&mut self) {
        let min_x = self.units.iter().map(|u| u.x).min().unwrap();
        let min_z = self.units.iter().map(|u| u.z).min().unwrap();
        for (ui, unit) in self.units.iter_mut().enumerate() {
            unit.first_spawnpoint = self.spawnpoints.len() as u32;
            let cu = unit.unit;
            let base_x = (unit.x as f32 + (cu.width as f32 / 2.0)) * 170.0;
            let base_z = (unit.z as f32 + (cu.height as f32 / 2.0)) * 170.0;
            for sp in cu.spawnpoints.iter() {
                let (x, z) = match cu.rotation {
                    0 => (base_x + sp.pos[0], base_z + sp.pos[2]),
                    1 => (base_x - sp.pos[2], base_z + sp.pos[0]),
                    2 => (base_x - sp.pos[0], base_z - sp.pos[2]),
                    _ => (base_x + sp.pos[2], base_z - sp.pos[0]),
                };
                let mut pos = Point([x, sp.pos[1], z]);
                pos[0] -= (min_x as f32) * 170.0;
                pos[2] -= (min_z as f32) * 170.0;
                self.spawnpoints.push(PlacedSpawnPoint {
                    spawnpoint: sp,
                    pos,
                    angle: (sp.angle_degrees - cu.rotation as f32 * 90.0) % 360.0,
                    unit: ui as u32,
                    hole_score: 0,
                    treasure_score: 0,
                    contains: Vec::new(),
                });
            }
            unit.x -= min_x;
            unit.z -= min_z;
        }
        for door in self.doors.iter_mut() {
            door.x -= min_x;
            door.z -= min_z;
        }
    }

    fn place_ship(&mut self) {
        let candidates: Vec<u32> = self.units[0].spawnpoint_indices().filter(|&i| self.spawnpoints[i].group() == 7).map(|i| i as u32).collect();
        let chosen = candidates[self.rng.rand_int(candidates.len() as u32) as usize];
        self.spawnpoints[chosen as usize].contains.push(SpawnObject::Ship);
        self.ship = Some(chosen);
    }

    fn ship_pos(&self) -> Point {
        self.spawnpoints[self.ship.unwrap() as usize].pos
    }

    /// Unit and door scores: roughly the distance from the ship plus the teki along the way.
    fn set_score(&mut self) {
        for unit in self.units.iter_mut() {
            unit.total_score = -1;
            unit.teki_score = 0;
        }
        for door in self.doors.iter_mut() {
            door.door_score = -1;
            door.seam_teki_score = 0;
        }

        for sp in self.spawnpoints.iter() {
            for so in sp.contains.iter() {
                match so {
                    SpawnObject::Teki(TekiInfo { group: 1, .. }, _) => self.units[sp.unit as usize].teki_score += 10,
                    SpawnObject::Teki(TekiInfo { group: 0, .. }, _) => self.units[sp.unit as usize].teki_score += 2,
                    _ => {}
                }
            }
        }
        for door in self.doors.iter_mut() {
            if let Some(SpawnObject::Teki(..)) = door.seam {
                door.seam_teki_score += 5;
            }
        }

        self.units[0].total_score = self.units[0].teki_score;
        for d in self.units[0].door_indices() {
            let score = self.units[0].total_score + 1 + self.doors[d].seam_teki_score;
            self.doors[d].door_score = score;
            let adj = self.doors[d].adjacent.expect("door isn't attached") as usize;
            self.doors[adj].door_score = score;
            let adj_unit = self.doors[adj].unit as usize;
            let candidate = score + self.units[adj_unit].teki_score;
            self.lower_total_score(adj_unit, candidate);
        }

        // Set scores outwards by repeatedly picking the smallest new door score. The game stops
        // after 500 steps (RandMapScore::setMapUnitScore) and only considers scores below 12,800,000.
        for _ in 0..500 {
            let mut selected = None;
            let mut selected_score = 12_800_000;
            for unit in self.units.iter() {
                for start in unit.door_indices() {
                    let start_door = &self.doors[start];
                    if start_door.door_score < 0 {
                        continue;
                    }
                    for link in start_door.door.door_links.iter() {
                        let other = unit.first_door as usize + link.door_id;
                        let other_door = &self.doors[other];
                        if other_door.door_score >= 0 {
                            continue;
                        }
                        let potential = (link.distance / 10.0) as i32
                            + unit.teki_score * i32::from(link.tekiflag)
                            + other_door.seam_teki_score
                            + start_door.door_score;
                        if potential < selected_score {
                            selected_score = potential;
                            selected = Some(other);
                        }
                    }
                }
            }
            let Some(selected) = selected else { break };
            self.doors[selected].door_score = selected_score;
            let adj = self.doors[selected].adjacent.expect("door isn't attached") as usize;
            self.doors[adj].door_score = selected_score;
            let adj_unit = self.doors[adj].unit as usize;
            let candidate = selected_score + self.units[adj_unit].teki_score;
            self.lower_total_score(adj_unit, candidate);
        }
    }

    /// RandMapScore::setMapNodeScore: keeps the lower score; -1 means unscored.
    fn lower_total_score(&mut self, unit: usize, score: i32) {
        let u = &mut self.units[unit];
        if u.total_score < 0 || u.total_score > score {
            u.total_score = score;
        }
    }

    // ---------------------------------------------------------------------------------
    // Hole, geyser and teki

    fn place_hole(&mut self, to_place: SpawnObject<'a>) {
        let challenge = self.ci.is_challenge_mode();
        let ship_pos = self.ship_pos();
        let ship_unit = self.spawnpoints[self.ship.unwrap() as usize].unit as usize;
        let mut candidates: Vec<usize> = Vec::new();
        for room_type in [RoomType::Room, RoomType::DeadEnd, RoomType::Hallway] {
            // Hallways are only used if nothing else is available.
            if room_type == RoomType::Hallway && !candidates.is_empty() {
                continue;
            }
            for (ui, unit) in self.units.iter().enumerate() {
                if unit.unit.room_type != room_type {
                    continue;
                }
                let score = if challenge {
                    let root = if unit.total_score > 0 { math::sqrt(unit.total_score as f32) } else { 0.0 };
                    root as i32 + 10
                } else {
                    unit.total_score
                };
                for si in unit.spawnpoint_indices() {
                    let sp = &mut self.spawnpoints[si];
                    if !sp.contains.is_empty() {
                        continue;
                    }
                    let group = sp.group();
                    // The 150-unit distance from the pod only applies inside the pod's own unit
                    // (RandMapScore::isFixObjSet).
                    let usable = (group == 4 && (ui != ship_unit || ship_pos.p2_dist(&sp.pos) >= 150.0)) || group == 9;
                    if usable {
                        sp.hole_score = score;
                        candidates.push(si);
                    }
                }
            }
        }
        if candidates.is_empty() {
            return;
        }

        let chosen = if challenge {
            let weights: Vec<u32> = candidates.iter().map(|&i| self.spawnpoints[i].hole_score as u32).collect();
            candidates[self.rng.rand_index_weight(&weights).unwrap()]
        } else {
            let best = candidates.iter().map(|&i| self.spawnpoints[i].hole_score).max().unwrap();
            let top: Vec<usize> = candidates.into_iter().filter(|&i| self.spawnpoints[i].hole_score == best).collect();
            top[self.rng.rand_int(top.len() as u32) as usize]
        };
        match to_place {
            SpawnObject::Hole(_) => self.hole = Some(chosen as u32),
            _ => self.geyser = Some(chosen as u32),
        }
        self.spawnpoints[chosen].contains.push(to_place);
    }

    /// Seam teki (group 5) on doors between units, preferring rooms 100 to 1 over hallways.
    fn place_seam_teki(&mut self) {
        for num_spawned in 0..self.slots[5] {
            let mut spots = Vec::new();
            let mut weights = Vec::new();
            for unit in self.units.iter() {
                if unit.unit.room_type == RoomType::DeadEnd {
                    continue;
                }
                for d in unit.door_indices() {
                    if self.doors[d].seam.is_some() {
                        continue;
                    }
                    spots.push(d);
                    weights.push(if unit.unit.room_type == RoomType::Room { 100 } else { 1 });
                }
            }
            let chosen = if spots.is_empty() { None } else { Some(spots[self.rng.rand_index_weight(&weights).unwrap()]) };
            // Picking the teki uses the RNG even when there's no spot.
            let teki = choose_rand_teki(&mut self.rng, self.ci, 5, num_spawned);
            match (chosen, teki) {
                (Some(d), Some(teki)) => {
                    let object = SpawnObject::Teki(teki, Point::ZERO);
                    self.doors[d].seam = Some(object);
                    let adj = self.doors[d].adjacent.expect("door isn't attached") as usize;
                    self.doors[adj].seam = Some(object);
                    self.placed_teki += 1;
                }
                _ => break,
            }
        }
    }

    /// Spawn points in rooms of `group` that are empty and far enough from the ship, hole and geyser.
    fn far_room_spawnpoints(&self, group: u16, from_ship: f32, from_exits: f32) -> Vec<usize> {
        let ship = self.ship_pos();
        let hole = self.hole.map(|i| self.spawnpoints[i as usize].pos);
        let geyser = self.geyser.map(|i| self.spawnpoints[i as usize].pos);
        self.units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::Room)
            .flat_map(|u| u.spawnpoint_indices())
            .filter(|&i| {
                let sp = &self.spawnpoints[i];
                sp.group() == group
                    && sp.contains.is_empty()
                    && ship.p2_dist(&sp.pos) >= from_ship
                    && hole.is_none_or(|h| h.p2_dist(&sp.pos) >= from_exits)
                    && geyser.is_none_or(|g| g.p2_dist(&sp.pos) >= from_exits)
            })
            .collect()
    }

    /// Places single teki of a group at random spots from `spots`, one per spot.
    fn place_single_teki(&mut self, group: u32, mut spots: Vec<usize>, count: u32) {
        for num_spawned in 0..count {
            let chosen = if spots.is_empty() { None } else { Some(spots.remove(self.rng.rand_int(spots.len() as u32) as usize)) };
            // Picking the teki uses the RNG even when there's no spot.
            let teki = choose_rand_teki(&mut self.rng, self.ci, group, num_spawned);
            match (chosen, teki) {
                (Some(i), Some(teki)) => {
                    self.spawnpoints[i].contains.push(SpawnObject::Teki(teki, Point::ZERO));
                    self.placed_teki += 1;
                }
                _ => break,
            }
        }
    }

    /// Special teki (group 8): 300 from the ship, 150 from the hole and geyser.
    fn place_special_teki(&mut self) {
        let spots = self.far_room_spawnpoints(8, 300.0, 150.0);
        self.place_single_teki(8, spots, self.slots[8]);
    }

    /// Hard teki (group 1): 300 from the ship, 200 from the hole and geyser.
    fn place_hard_teki(&mut self) {
        let spots = self.far_room_spawnpoints(1, 300.0, 200.0);
        self.place_single_teki(1, spots, self.slots[1]);
    }

    /// Easy teki (group 0): bunches of several teki, 300 from the ship.
    fn place_easy_teki(&mut self) {
        let ship = self.ship_pos();
        let mut spots: Vec<usize> = self
            .units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::Room)
            .flat_map(|u| u.spawnpoint_indices())
            .filter(|&i| {
                let sp = &self.spawnpoints[i];
                sp.group() == 0 && sp.contains.is_empty() && ship.p2_dist(&sp.pos) >= 300.0
            })
            .collect();

        let mut num_spawned = 0;
        while num_spawned < self.slots[0] {
            let (mut min_num, mut max_num) = (0u16, 0u16);
            let chosen = if spots.is_empty() {
                None
            } else {
                let idx = self.rng.rand_int(spots.len() as u32) as usize;
                let sp = self.spawnpoints[spots[idx]].spawnpoint;
                (min_num, max_num) = (sp.min_num, sp.max_num);
                Some(spots.remove(idx))
            };
            let teki = choose_rand_teki(&mut self.rng, self.ci, 0, num_spawned);

            // How many to spawn in this bunch.
            let spawn_in_room = if num_spawned < self.min_teki_0 {
                let mut cumulative_min = 0;
                for t in self.ci.teki_group(0) {
                    cumulative_min += t.minimum_amount;
                    if cumulative_min > num_spawned {
                        break;
                    }
                }
                cumulative_min - num_spawned
            } else {
                // Wraps like the reference's release build if more teki were placed than allowed.
                self.ci.max_main_objects.wrapping_sub(self.placed_teki)
            };
            max_num = min(max_num, spawn_in_room as u16);
            let num_to_spawn = if max_num <= min_num { max_num as u32 } else { min_num as u32 + self.rng.rand_int((max_num - min_num + 1) as u32) };
            if num_to_spawn == 0 {
                break;
            }

            let (Some(spot), Some(teki)) = (chosen, teki) else { break };
            let radius_max = self.spawnpoints[spot].spawnpoint.radius;
            let mut offsets: Vec<Point> = Vec::with_capacity(num_to_spawn as usize);
            for _ in 0..num_to_spawn {
                let radius = radius_max * self.rng.rand_f32();
                let angle = std::f32::consts::PI * 2.0 * self.rng.rand_f32();
                // sin and cos are swapped compared to the usual convention.
                offsets.push(Point([angle.sin() * radius, 0.0, angle.cos() * radius]));
                num_spawned += 1;
                self.placed_teki += 1;
            }
            // Push teki in the bunch apart.
            for _ in 0..5 {
                for a in 0..offsets.len() {
                    for b in 0..offsets.len() {
                        if a == b {
                            continue;
                        }
                        let delta = offsets[a] - offsets[b];
                        let dist = offsets[a].p2_dist(&offsets[b]);
                        if dist > 0.0 && dist < 35.0 {
                            let multiplier = 0.5 * (35.0 - dist) / dist;
                            offsets[a] += delta * multiplier;
                            offsets[b] -= delta * multiplier;
                        }
                    }
                }
            }
            self.spawnpoints[spot].contains.extend(offsets.into_iter().map(|o| SpawnObject::Teki(teki, o)));
        }
    }

    /// Plants (group 6), placed until their minimum amounts are met.
    fn place_plants(&mut self) {
        let spots: Vec<usize> = (0..self.spawnpoints.len())
            .filter(|&i| self.spawnpoints[i].group() == 6 && self.spawnpoints[i].contains.is_empty())
            .collect();
        let count: u32 = self.ci.teki_group(6).map(|t| t.minimum_amount).sum();
        self.place_single_teki(6, spots, count);
    }

    // ---------------------------------------------------------------------------------
    // Treasures, alcoves, gates and onions

    fn place_items(&mut self) {
        let challenge = self.ci.is_challenge_mode();
        for num_spawned in 0..self.ci.max_treasures {
            let mut spots: Vec<usize> = Vec::new();
            let mut weights: Vec<i32> = Vec::new();
            for unit in self.units.iter() {
                if unit.unit.room_type == RoomType::Room {
                    let sps = &self.spawnpoints[unit.spawnpoint_indices()];
                    let items_here = sps.iter().flat_map(|sp| sp.contains.iter()).filter(|so| matches!(so, SpawnObject::Item(_))).count();
                    let treasure_spots = sps.iter().filter(|sp| sp.group() == 2).count();
                    for si in unit.spawnpoint_indices() {
                        let sp = &self.spawnpoints[si];
                        if sp.group() != 2 || !sp.contains.is_empty() {
                            continue;
                        }
                        let w = if challenge {
                            1 + unit.total_score / treasure_spots as i32
                        } else {
                            unit.total_score / (1 + items_here) as i32
                        };
                        spots.push(si);
                        weights.push(w);
                    }
                } else if unit.unit.unit_folder_name.contains("item") {
                    let si = unit
                        .spawnpoint_indices()
                        .find(|&i| self.spawnpoints[i].group() == 9)
                        .expect("alcove has no alcove spawn point");
                    if !self.spawnpoints[si].contains.is_empty() {
                        continue;
                    }
                    spots.push(si);
                    weights.push(if challenge { 1 + unit.total_score * 10 } else { 1 + unit.total_score });
                }
            }
            for (&si, &w) in spots.iter().zip(&weights) {
                self.spawnpoints[si].treasure_score = w;
            }

            let chosen = if spots.is_empty() {
                None
            } else if challenge {
                self.rng.rand_index_weight_signed(&weights).map(|i| spots[i])
            } else {
                let best = *weights.iter().max().unwrap();
                let top: Vec<usize> = spots.iter().zip(&weights).filter(|(_, w)| **w == best).map(|(s, _)| *s).collect();
                Some(top[self.rng.rand_int(top.len() as u32) as usize])
            };
            let item = choose_rand_item(&mut self.rng, self.ci, num_spawned);
            if let (Some(si), Some(item)) = (chosen, item) {
                self.spawnpoints[si].contains.push(SpawnObject::Item(item));
            }
        }
    }

    /// Alcove spawn points of item alcoves, in unit order.
    fn alcove_spots(&self) -> Vec<usize> {
        self.units
            .iter()
            .filter(|u| u.unit.room_type == RoomType::DeadEnd && u.unit.unit_folder_name.contains("item"))
            .map(|u| u.spawnpoint_indices().find(|&i| self.spawnpoints[i].group() == 9).expect("alcove has no alcove spawn point"))
            .collect()
    }

    fn place_cap_teki(&mut self) {
        // Grounded cap teki. Not random, which is why e.g. the Mitites on HoB4 are predictable.
        let mut num_spawned = 0;
        for si in self.alcove_spots() {
            if !self.spawnpoints[si].contains.is_empty() {
                continue;
            }
            if let Some((teki, n)) = choose_rand_cap_teki(&mut self.rng, self.ci, num_spawned, false) {
                self.spawnpoints[si].contains.push(SpawnObject::CapTeki(teki, n));
                num_spawned += n;
            }
        }

        // Falling cap teki, which can land on other cap teki but not Candypop Buds or exits.
        num_spawned = 0;
        for si in self.alcove_spots() {
            let blocked = self.spawnpoints[si].contains.iter().any(|so| match so {
                SpawnObject::CapTeki(teki, _) => teki.is_candypop() || teki.is_falling(),
                SpawnObject::Hole(_) | SpawnObject::Geyser(_) => true,
                _ => false,
            });
            if blocked {
                continue;
            }
            if let Some((teki, n)) = choose_rand_cap_teki(&mut self.rng, self.ci, num_spawned, true) {
                self.spawnpoints[si].contains.push(SpawnObject::CapTeki(teki, n));
                num_spawned += n;
            }
        }
    }

    fn place_gates(&mut self) {
        for _ in 0..self.ci.max_gates {
            let weights: Vec<u32> = self.ci.gate_info.iter().map(|g| g.spawn_distribution_weight).collect();
            let gate = if weights.is_empty() { None } else { Some(&self.ci.gate_info[self.rng.rand_index_weight(&weights).unwrap()]) };
            let spot = self.gate_spot();
            if let (Some(gate), Some(d)) = (gate, spot) {
                let object = SpawnObject::Gate(gate, self.doors[d].door.direction);
                self.doors[d].seam = Some(object);
                let adj = self.doors[d].adjacent.expect("door isn't attached") as usize;
                self.doors[adj].seam = Some(object);
            }
        }
    }

    /// Where the next gate goes. The game tries four ways in order:
    /// 1. in front of an item alcove holding a treasure, exit or grounded cap teki;
    /// 2. the lowest-scored door of each room (except the ship's), if free;
    /// 3. (80% of the time) a room door weighted towards low door scores;
    /// 4. any free door.
    fn gate_spot(&mut self) -> Option<usize> {
        // 1
        let mut spots = Vec::new();
        for unit in self.units.iter() {
            if unit.unit.room_type != RoomType::DeadEnd || !unit.unit.unit_folder_name.contains("item") {
                continue;
            }
            // The alcove's own spot is its group 9 spawn point; some romhack alcoves list a plant spot first.
            let Some(si) = unit.spawnpoint_indices().find(|&i| self.spawnpoints[i].group() == 9) else { continue };
            let contains = &self.spawnpoints[si].contains;
            let filled = contains.iter().any(|so| match so {
                SpawnObject::CapTeki(teki, _) => !teki.is_falling(),
                SpawnObject::Item(_) | SpawnObject::Hole(_) | SpawnObject::Geyser(_) => true,
                _ => false,
            });
            let falling_candypop = contains.iter().any(|so| matches!(so, SpawnObject::CapTeki(teki, _) if teki.is_candypop() && teki.is_falling()));
            if !filled || falling_candypop {
                continue;
            }
            let d = unit.first_door as usize;
            if self.doors[d].seam.is_none() {
                spots.push(d);
            }
        }
        if !spots.is_empty() {
            return Some(spots[self.rng.rand_int(spots.len() as u32) as usize]);
        }

        // 2
        let ship_unit = self.spawnpoints[self.ship.unwrap() as usize].unit as usize;
        for (ui, unit) in self.units.iter().enumerate() {
            if unit.unit.room_type != RoomType::Room || ui == ship_unit {
                continue;
            }
            let mut min_score = 128_000;
            let mut min_door = None;
            for d in unit.door_indices() {
                if self.doors[d].door_score < min_score {
                    min_score = self.doors[d].door_score;
                    min_door = Some(d);
                }
            }
            if let Some(d) = min_door
                && min_score < 128_000
                && self.doors[d].seam.is_none()
            {
                return Some(d);
            }
        }

        // 3
        let mut weights = Vec::new();
        if self.rng.rand_f32() < 0.8 {
            let room_doors = || {
                self.units
                    .iter()
                    .filter(|u| u.unit.room_type == RoomType::Room)
                    .flat_map(|u| u.door_indices())
                    .filter(|&d| self.doors[d].seam.is_none())
            };
            let max_score = room_doors().map(|d| self.doors[d].door_score).fold(0, max);
            for d in room_doors() {
                spots.push(d);
                // At least 1, since max_score covers these same doors.
                weights.push((max_score + 1 - self.doors[d].door_score) as u32);
            }
            if !spots.is_empty() {
                return Some(spots[self.rng.rand_index_weight(&weights).unwrap()]);
            }
        }

        // 4
        for unit in self.units.iter() {
            for d in unit.door_indices() {
                if self.doors[d].seam.is_some() {
                    continue;
                }
                spots.push(d);
                let n = unit.unit.doors.len();
                weights.push(if unit.unit.room_type == RoomType::Hallway { (10 / n) as u32 } else { n as u32 });
            }
        }
        if !spots.is_empty() {
            return Some(spots[self.rng.rand_index_weight(&weights).unwrap()]);
        }
        None
    }

    /// Colossal Caverns places three onions at evenly spaced rooms with a ship spot.
    fn place_onions(&mut self) {
        const NUM_ONIONS: usize = 3;
        let has_ship_spot = |u: &PlacedUnit| u.unit.room_type == RoomType::Room && u.unit.spawnpoints.iter().any(|sp| sp.group == 7);
        let valid_rooms = self.units.iter().filter(|u| has_ship_spot(u)).count();
        let interval = max(valid_rooms / NUM_ONIONS, 1);
        let mut color = 0;
        let mut room_num = 0;
        for ui in 0..self.units.len() {
            if self.units[ui].unit.room_type != RoomType::Room {
                continue;
            }
            if let Some(si) = self.units[ui].spawnpoint_indices().find(|&i| self.spawnpoints[i].group() == 7) {
                room_num += 1;
                if room_num == (color + 1) * interval {
                    self.spawnpoints[si].contains.push(SpawnObject::Onion((NUM_ONIONS - 1 - color) as u32));
                    color += 1;
                }
            }
            if color >= NUM_ONIONS {
                break;
            }
        }
    }
}

/// Grid position of `door` when `unit` is placed with its corner at (`x`, `z`).
#[inline]
pub(crate) fn door_position(unit: &CaveUnit, door: &crate::caveinfo::DoorUnit, x: i32, z: i32) -> (i32, i32) {
    let offset = door.side_lateral_offset as i32;
    match door.direction {
        0 => (x + offset, z),
        1 => (x + unit.width as i32, z + offset),
        2 => (x + offset, z + unit.height as i32),
        _ => (x, z + offset),
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn boxes_overlap(x1: i32, z1: i32, w1: u16, h1: u16, x2: i32, z2: i32, w2: u16, h2: u16) -> bool {
    !((x1 + w1 as i32 <= x2 || x2 + w2 as i32 <= x1) || (z1 + h1 as i32 <= z2 || z2 + h2 as i32 <= z1))
}

/// The game's teki choice: teki fill their minimum amounts in order, then the rest are
/// picked by weight. Uses the RNG only for weighted picks.
fn choose_rand_teki<'a>(rng: &mut Rng, ci: &'a CaveInfo, group: u32, num_spawned: u32) -> Option<&'a TekiInfo> {
    let mut cumulative = 0;
    let mut filler = Vec::new();
    let mut weights = Vec::new();
    for teki in ci.teki_group(group) {
        cumulative += teki.minimum_amount;
        if num_spawned < cumulative {
            return Some(teki);
        }
        if teki.filler_distribution_weight > 0 {
            filler.push(teki);
            weights.push(teki.filler_distribution_weight);
        }
    }
    if filler.is_empty() { None } else { Some(filler[rng.rand_index_weight(&weights).unwrap()]) }
}

fn choose_rand_item<'a>(rng: &mut Rng, ci: &'a CaveInfo, num_spawned: u32) -> Option<&'a ItemInfo> {
    let mut cumulative = 0u32;
    let mut filler = Vec::new();
    let mut weights = Vec::new();
    for item in ci.item_info.iter() {
        cumulative += item.min_amount as u32;
        if num_spawned < cumulative {
            return Some(item);
        }
        if item.filler_distribution_weight > 0 {
            filler.push(item);
            weights.push(item.filler_distribution_weight);
        }
    }
    if filler.is_empty() { None } else { Some(filler[rng.rand_index_weight(&weights).unwrap()]) }
}

/// Picks an alcove teki and how many spawn (2 for group 0, else 1).
fn choose_rand_cap_teki<'a>(rng: &mut Rng, ci: &'a CaveInfo, num_spawned: u32, falling: bool) -> Option<(&'a CapInfo, u32)> {
    let mut cumulative = 0;
    let mut filler = Vec::new();
    let mut weights = Vec::new();
    for teki in ci.cap_info.iter().filter(|t| if falling { t.is_falling() && !t.is_candypop() } else { !t.is_falling() || t.is_candypop() }) {
        cumulative += teki.minimum_amount;
        if num_spawned < cumulative {
            return Some((teki, if teki.group == 0 && num_spawned + 1 < cumulative { 2 } else { 1 }));
        }
        if teki.filler_distribution_weight > 0 {
            filler.push(teki);
            weights.push(teki.filler_distribution_weight);
        }
    }
    if filler.is_empty() {
        // The game still uses the RNG here.
        rng.rand_raw();
        None
    } else {
        let teki = filler[rng.rand_index_weight(&weights).unwrap()];
        Some((teki, if teki.group == 0 { 2 } else { 1 }))
    }
}
