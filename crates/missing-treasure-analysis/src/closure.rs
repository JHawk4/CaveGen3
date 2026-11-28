//! Fast tests for hunting seeds whose map closes with only a few rooms.

use cavegen3::{
    caveinfo::{CaveInfo, RoomType},
    generate::internals::MapGen,
    rng::Rng,
};

/// Whether a seed's map closes with at most `max_rooms` rooms placed, stopping as soon as one
/// more room is placed: a fast test for hunting maps that close with only a few rooms.
pub fn closes_with_rooms(rng: Rng, caveinfo: &CaveInfo, max_rooms: u32) -> bool {
    let mut g = MapGen::new(rng, caveinfo);
    if !g.place_start() {
        return true;
    }
    for steps in 1..=500 {
        if g.rooms_placed() > max_rooms {
            return false;
        }
        if !g.placement_step(steps) {
            return g.rooms_placed() <= max_rooms;
        }
    }
    false
}

/// Rooms placed after the start room and `steps` placement steps (for checking
/// [`ClosureFilter`]).
pub fn rooms_after_steps(rng: Rng, caveinfo: &CaveInfo, steps: u32) -> u32 {
    let mut g = MapGen::new(rng, caveinfo);
    if g.place_start() {
        for step in 1..=steps {
            if !g.placement_step(step) {
                break;
            }
        }
    }
    g.rooms_placed()
}

/// A fast test that rules out most seeds when hunting maps that close with only the start room.
///
/// The first placement step's corridor roll always lands on the same `rand()` call, and if it
/// says "room first", a room is placed: [`ClosureFilter::new`] checks that some room fits at
/// every door of every possible start room. That is one multiply per seed.
///
/// For the seeds left, [`ClosureFilter::rejects`] replays the first step (a hallway) and the
/// second step's door pick and roll on small arrays, with the map geometry looked up in tables
/// made by the real generator, and rules the seed out if the second step certainly places a room.
#[derive(Clone, Debug)]
pub struct ClosureFilter {
    generator: cavegen3::Generator,
    /// Steps from the seed to just before the first step's corridor roll.
    jump: (u64, u64),
    /// The first roll is below this for "hallway first" (the destination is the start room).
    corridor_probability: f32,
    second: Option<Box<SecondStep>>,
}

/// What [`ClosureFilter`] needs to replay the first step when it places a hallway.
#[derive(Clone, Debug)]
struct SecondStep {
    corridor_probability: f32,
    max_doors: usize,
    max_unit_doors: usize,
    /// Steps over the cap list shuffle, the teki slots and the start room's reshuffle, and the
    /// hallway list reshuffle after a hallway is placed.
    skip_caps: (u64, u64),
    skip_teki_start: (u64, u64),
    skip_corridor_backs: (u64, u64),
    /// Room units in list order before shuffling, and whether each can be the start room.
    rooms: Vec<(u16, bool)>,
    /// Hallway units in list order before shuffling, with their door counts.
    corridors: Vec<(u16, usize)>,
    /// Per start room: its id, its open door count and where its entries in `after` begin.
    starts: Vec<(u16, usize, usize)>,
    /// After a start room and a hallway attached at (start door, hallway, hallway door): the
    /// open doors in order as (on a room, some room fits there), and whether the map is 36 wide.
    /// None if that hallway doesn't fit there.
    after: Vec<Option<(Vec<(bool, bool)>, bool)>>,
}

/// `rand_backs` on a small array: moves a random element to the back, once per element.
#[inline]
fn rand_backs_slice(rng: &mut Rng, list: &mut [u16]) {
    let n = list.len();
    for _ in 0..n {
        let i = rng.rand_int(n as u32) as usize;
        list[i..].rotate_left(1);
    }
}

impl ClosureFilter {
    /// None if the shortcut doesn't hold for this sublevel.
    pub fn new(caveinfo: &CaveInfo, generator: cavegen3::Generator) -> Option<Self> {
        // Where the roll falls: count calls with tapes of different random values.
        let mut roll_call = None;
        for k in 0..200u64 {
            let mut g = MapGen::new(Rng::tape(generator, Vec::new(), 0xF117_0000 + k), caveinfo);
            if !g.place_start() || caveinfo.num_rooms < 2 || g.diameter_36() {
                return None;
            }
            let calls = g.rng().tape_ref().unwrap().pos + 1; // the door pick comes first
            if *roll_call.get_or_insert(calls) != calls {
                return None;
            }
        }
        let roll_call = roll_call? as u64;
        // Some room must fit at every door of every room the map can start with.
        let g = MapGen::new(Rng::with(generator, 0), caveinfo);
        for &start in g.room_queue() {
            if !caveinfo.cave_units[start as usize].has_start_spawnpoint() {
                continue;
            }
            let mut s = g.clone();
            s.place_unit_at_origin(start);
            if s.open_doors().is_empty() || !s.open_doors().iter().all(|&d| s.room_fits_at(d)) {
                return None;
            }
        }
        Some(ClosureFilter {
            generator,
            jump: Rng::jump_coefficients(generator, roll_call),
            // The destination is the start room, which doubles it.
            corridor_probability: caveinfo.corridor_probability * 2.0,
            second: SecondStep::new(caveinfo, generator, roll_call).map(Box::new),
        })
    }

    /// Whether the second step is checked too.
    pub fn checks_second_step(&self) -> bool {
        self.second.is_some()
    }

    /// True if the seed's first step certainly places a second room.
    #[inline]
    pub fn rejects_first_step(&self, seed: u64) -> bool {
        let mut rng = Rng::with(self.generator, seed);
        rng.jump(self.jump);
        rng.rand_f32() >= self.corridor_probability
    }

    /// True if the seed's first or second step certainly places a second room.
    #[inline]
    pub fn rejects(&self, seed: u64) -> bool {
        self.rejects_first_step(seed) || self.second.as_ref().is_some_and(|s| s.rejects(self.generator, seed))
    }
}

impl SecondStep {
    fn new(caveinfo: &CaveInfo, generator: cavegen3::Generator, roll_call: u64) -> Option<Self> {
        if caveinfo.ultra_randomizer {
            return None;
        }
        let units = &caveinfo.cave_units;
        let of = |t: RoomType| units.iter().enumerate().filter(move |u| u.1.room_type == t).map(|u| u.0 as u16);
        let rooms: Vec<(u16, bool)> = of(RoomType::Room).map(|id| (id, units[id as usize].has_start_spawnpoint())).collect();
        let corridors: Vec<(u16, usize)> = of(RoomType::Hallway).map(|id| (id, units[id as usize].num_doors)).collect();
        let caps = of(RoomType::DeadEnd).count() as u64;
        let max_doors = caveinfo.max_num_doors_single_unit();
        if corridors.is_empty() || rooms.len() > 64 || corridors.len() > 64 || max_doors > 16 {
            return None;
        }
        // The calls before the first door pick: the three list shuffles, then the rest.
        let shuffles = caps + rooms.len() as u64 + corridors.len() as u64;
        let teki_start = (roll_call - 1).checked_sub(shuffles)?;
        let max_unit_doors = corridors.iter().map(|c| c.1).max().unwrap();

        let g = MapGen::new(Rng::with(generator, 0), caveinfo);
        let mut starts = Vec::new();
        let mut after = Vec::new();
        for &(start, can_start) in &rooms {
            if !can_start {
                continue;
            }
            let mut s = g.clone();
            s.place_unit_at_origin(start);
            let open = s.open_doors();
            starts.push((start, open.len(), after.len()));
            for &d in open {
                for &(id, num_doors) in &corridors {
                    for di in 0..max_unit_doors {
                        let placed = if di < num_doors { s.attach(d, id, di) } else { None };
                        after.push(placed.map(|h| {
                            let open = h.open_doors().iter().map(|&od| (h.door_on_room(od), h.room_fits_at(od))).collect();
                            (open, h.diameter_36())
                        }));
                    }
                }
            }
        }
        Some(SecondStep {
            corridor_probability: caveinfo.corridor_probability,
            max_doors,
            max_unit_doors,
            skip_caps: Rng::jump_coefficients(generator, caps),
            skip_teki_start: Rng::jump_coefficients(generator, teki_start),
            skip_corridor_backs: Rng::jump_coefficients(generator, corridors.len() as u64),
            rooms,
            corridors,
            starts,
            after,
        })
    }

    /// Replays the first step for a seed whose first roll says "hallway first" (the caller checks
    /// that), then the second step's door pick and roll.
    fn rejects(&self, generator: cavegen3::Generator, seed: u64) -> bool {
        let mut rng = Rng::with(generator, seed);
        rng.jump(self.skip_caps);
        // The start room: the first room in the shuffled list that can be one.
        let mut room_order = [0u16; 64];
        let room_order = &mut room_order[..self.rooms.len()];
        for (i, r) in room_order.iter_mut().enumerate() {
            *r = i as u16;
        }
        rand_backs_slice(&mut rng, room_order);
        let Some(&start_pos) = room_order.iter().find(|&&r| self.rooms[r as usize].1) else {
            return false;
        };
        let start_id = self.rooms[start_pos as usize].0;
        let Some(&(_, n0, base)) = self.starts.iter().find(|s| s.0 == start_id) else {
            return false;
        };
        let mut corridor_order = [0u16; 64];
        let corridor_order = &mut corridor_order[..self.corridors.len()];
        for (i, c) in corridor_order.iter_mut().enumerate() {
            *c = i as u16;
        }
        rand_backs_slice(&mut rng, corridor_order);
        rng.jump(self.skip_teki_start);
        let door = rng.rand_int(n0 as u32) as usize;
        rng.rand_f32(); // the first roll: "hallway first"

        // shuffle_corridor_priority, with the start room's doors open.
        let mut priority = [0usize; 16];
        let priority = &mut priority[..self.max_doors];
        for (i, p) in priority.iter_mut().enumerate() {
            *p = if n0 < 4 { self.max_doors - i } else { i + 1 };
        }
        if (4..10).contains(&n0) {
            rng.rand_swaps(priority);
        }
        // Hallways in that order, each tried by its doors in random order.
        let per_door = self.corridors.len() * self.max_unit_doors;
        let mut placed = None;
        'outer: for &num_doors in priority.iter() {
            for &c in corridor_order.iter() {
                if self.corridors[c as usize].1 != num_doors {
                    continue;
                }
                let mut order = [0usize; 16];
                let order = &mut order[..num_doors];
                for (i, d) in order.iter_mut().enumerate() {
                    *d = i;
                }
                rng.rand_swaps(order);
                for &di in order.iter() {
                    if let Some(a) = &self.after[base + door * per_door + c as usize * self.max_unit_doors + di] {
                        placed = Some(a);
                        break 'outer;
                    }
                }
            }
        }
        let Some((open, diameter_36)) = placed else {
            return false;
        };
        rng.jump(self.skip_corridor_backs);
        if open.is_empty() {
            return false;
        }
        // The second step: door pick and roll.
        let (on_room, room_fits) = open[rng.rand_int(open.len() as u32) as usize];
        let mut cp = if *diameter_36 { 0.0 } else { self.corridor_probability };
        if on_room {
            cp *= 2.0;
        }
        room_fits && rng.rand_f32() >= cp
    }
}

/// Seconds spent in each stage of [`closes_with_rooms`] over `n` seeds (setup with the unit list
/// shuffles and teki slots, start room, placement steps), and the mean number of placement steps:
/// for speeding up hunts.
pub fn profile_closes_with_rooms(caveinfo: &CaveInfo, generator: cavegen3::Generator, n: u64, max_rooms: u32) -> ([f64; 3], f64) {
    let mut t = [0.0; 3];
    let mut total_steps = 0u64;
    for i in 0..n {
        let seed = i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x0F1E_2D3C_4B5A_6978;
        let c0 = std::time::Instant::now();
        let mut g = MapGen::new(Rng::with(generator, seed), caveinfo);
        let c1 = std::time::Instant::now();
        g.place_start();
        let c2 = std::time::Instant::now();
        for steps in 1..=500 {
            total_steps += 1;
            if g.rooms_placed() > max_rooms || !g.placement_step(steps) {
                break;
            }
        }
        let c3 = std::time::Instant::now();
        t[0] += (c1 - c0).as_secs_f64();
        t[1] += (c2 - c1).as_secs_f64();
        t[2] += (c3 - c2).as_secs_f64();
    }
    (t, total_steps as f64 / n as f64)
}
