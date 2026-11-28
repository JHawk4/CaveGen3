//! The generator's state between placement steps, for analysis tools outside this crate (the
//! `missing-treasure-analysis` crate). Only built with the `internals` feature; not a stable API.

use super::*;

/// A map partway through generation.
#[derive(Clone)]
pub struct MapGen<'a>(Generator<'a>);

impl<'a> MapGen<'a> {
    /// Starts generating with `rng`: the unit lists are shuffled and the teki slots allocated,
    /// and no unit is placed yet.
    pub fn new(rng: Rng, caveinfo: &'a CaveInfo) -> Self {
        let mut g = Generator::new(rng, caveinfo);
        g.sort_units_and_allocate_teki();
        MapGen(g)
    }

    /// Places the first room; false if it has no doors.
    pub fn place_start(&mut self) -> bool {
        self.0.place_start()
    }

    /// One step of the placement loop (`steps` counts from 1); false once the map is closed.
    pub fn placement_step(&mut self, steps: u32) -> bool {
        self.0.placement_step(steps)
    }

    /// Places the start room and every other unit; returns the number of placement steps.
    pub fn place_map_units(&mut self) -> u32 {
        self.0.place_map_units()
    }

    /// Everything after map unit placement: ship, exits, teki, treasures, gates.
    pub fn finish(&mut self) {
        self.0.finish()
    }

    /// [`MapGen::finish`] up to and including treasure placement.
    pub fn finish_through_items(&mut self) {
        let g = &mut self.0;
        g.create_spawnpoints_and_recenter();
        g.place_ship();
        g.set_score();
        if !g.ci.is_final_floor {
            g.place_hole(SpawnObject::Hole(g.ci.exit_plugged));
        }
        if g.ci.is_final_floor || g.ci.has_geyser {
            g.place_hole(SpawnObject::Geyser(g.ci.is_challenge_mode() && g.ci.is_final_floor));
        }
        g.place_seam_teki();
        g.place_special_teki();
        g.place_hard_teki();
        g.place_easy_teki();
        g.set_score();
        g.place_plants();
        g.place_items();
    }

    pub fn into_layout(self) -> Layout<'a> {
        self.0.into_layout()
    }

    pub fn caveinfo(&self) -> &'a CaveInfo {
        self.0.ci
    }

    pub fn rng(&self) -> &Rng {
        &self.0.rng
    }

    pub fn rng_mut(&mut self) -> &mut Rng {
        &mut self.0.rng
    }

    pub fn rooms_placed(&self) -> u32 {
        self.0.rooms_placed
    }

    /// Whether the map is 36 grid cells wide or tall (no more hallway-first rolls).
    pub fn diameter_36(&self) -> bool {
        self.0.diameter_36
    }

    pub fn units(&self) -> &[PlacedUnit<'a>] {
        &self.0.units
    }

    /// Filled in by [`MapGen::finish`] or [`MapGen::finish_through_items`].
    pub fn spawnpoints(&self) -> &[PlacedSpawnPoint<'a>] {
        &self.0.spawnpoints
    }

    /// The room units in their current priority order.
    pub fn room_queue(&self) -> &[u16] {
        &self.0.room_queue
    }

    /// Indices of the doors with nothing attached, in the game's order.
    pub fn open_doors(&self) -> &[u32] {
        &self.0.open
    }

    /// Whether door `d` belongs to a room unit.
    pub fn door_on_room(&self, d: u32) -> bool {
        let g = &self.0;
        g.units[g.doors[d as usize].unit as usize].unit.room_type == RoomType::Room
    }

    /// How many open doors a room could be attached to right now.
    pub fn roomable_doors(&self) -> usize {
        self.0.roomable_doors()
    }

    /// How many ways a room could be placed right now (open door, room unit, unit door).
    pub fn room_fits(&self) -> usize {
        let g = &self.0;
        g.open
            .iter()
            .map(|&d| {
                g.room_queue
                    .iter()
                    .map(|&id| (0..g.unit(id).num_doors).filter(|&di| g.try_place_unit_at(d, id, di).is_some()).count())
                    .sum::<usize>()
            })
            .sum()
    }

    /// Whether some room fits at open door `d`.
    pub fn room_fits_at(&self, d: u32) -> bool {
        let g = &self.0;
        g.room_queue.iter().any(|&id| (0..g.unit(id).num_doors).any(|di| g.try_place_unit_at(d, id, di).is_some()))
    }

    /// Places unit `id` (an index into the caveinfo's units) with its corner at the origin.
    pub fn place_unit_at_origin(&mut self, id: u16) {
        self.0.place_unit(Candidate { unit: id, x: 0, z: 0 });
    }

    /// This map with unit `id` attached by its door `unit_door` to open door `door`, if it fits.
    pub fn attach(&self, door: u32, id: u16, unit_door: usize) -> Option<Self> {
        let c = self.0.try_place_unit_at(door, id, unit_door)?;
        let mut m = self.clone();
        m.0.place_unit(c);
        Some(m)
    }
}
