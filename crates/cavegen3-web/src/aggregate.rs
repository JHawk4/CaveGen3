//! CaveGen's Aggregator: generate many seeds of a sublevel, group the layouts that look alike,
//! and count what spawns at each spot in each group.
//!
//! Positions are taken relative to an anchor unit and turned so the anchor has rotation 0, as
//! CaveGen does, so the same arrangement in any rotation lands in one group. The anchor is the
//! first unit placed (the room generation starts from), or for comedown grouping the unit the
//! pod is in.

use std::collections::HashMap;

use cavegen3::{Layout, SpawnObject, caveinfo::RoomType, point::Point};

/// How layouts are grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    /// The whole layout: every map unit and where it is (CaveGen's default).
    Layout,
    /// Only the rooms (CaveGen's -aggRooms).
    Rooms,
    /// Only the first room placed (CaveGen's -aggFirst).
    First,
    /// What you see when you land: the map units in a box around the pod.
    ComedownUnits,
    /// The comedown's map units plus the objects in it.
    Comedown,
}

impl Grouping {
    pub fn parse(s: &str) -> Result<Self, String> {
        Ok(match s {
            "layout" => Grouping::Layout,
            "rooms" => Grouping::Rooms,
            "first" => Grouping::First,
            "comedown-units" => Grouping::ComedownUnits,
            "comedown" => Grouping::Comedown,
            _ => return Err(format!("unknown grouping {s:?}")),
        })
    }

    fn is_comedown(self) -> bool {
        matches!(self, Grouping::ComedownUnits | Grouping::Comedown)
    }
}

/// Groups beyond this many are counted but not kept, to bound memory when nearly every layout
/// is different (whole-layout grouping on a big sublevel).
const MAX_GROUPS: usize = 20_000;
const EXAMPLE_SEEDS: usize = 8;

/// A map unit of a group, relative to the anchor's center, in game units.
#[derive(Debug, Clone)]
struct GroupUnit {
    name: String,
    game: String,
    x: f32,
    z: f32,
    /// Size in cells after rotation.
    w: u16,
    h: u16,
    rot: u16,
}

#[derive(Debug, Default)]
struct Group {
    count: u64,
    seeds: Vec<u32>,
    /// The units that define the group (from its first layout).
    units: Vec<GroupUnit>,
    /// The pod, for comedown groups.
    pod: Option<(f32, f32)>,
    /// What spawned at each spot: spot -> object -> count.
    spots: HashMap<(i32, i32), HashMap<String, u64>>,
    /// How often each half-cell position was covered by a map unit (keys are doubled cell
    /// coordinates of cell centers).
    cells: HashMap<(i32, i32), u64>,
}

pub struct Aggregator {
    pub sublevel: String,
    grouping: Grouping,
    /// Half the comedown box's width, in game units.
    half: f32,
    total: u64,
    dropped: u64,
    groups: HashMap<String, Group>,
}

/// Turns positions so the anchor unit has rotation 0, about the anchor's center.
struct Frame {
    cx: f32,
    cz: f32,
    rot: u16,
}

impl Frame {
    fn new(layout: &Layout, anchor: usize) -> Frame {
        let a = &layout.units[anchor];
        Frame {
            cx: (a.x as f32 + a.unit.width as f32 / 2.0) * 170.0,
            cz: (a.z as f32 + a.unit.height as f32 / 2.0) * 170.0,
            rot: a.unit.rotation % 4,
        }
    }

    /// Undoes `rot` quarter turns; a unit's rotation 1 maps (x, z) to (-z, x).
    fn apply(&self, x: f32, z: f32) -> (f32, f32) {
        let (mut x, mut z) = (x - self.cx, z - self.cz);
        for _ in 0..self.rot {
            (x, z) = (z, -x);
        }
        (x, z)
    }

    fn unit(&self, layout: &Layout, i: usize) -> GroupUnit {
        let u = &layout.units[i];
        let (x, z) = self.apply((u.x as f32 + u.unit.width as f32 / 2.0) * 170.0, (u.z as f32 + u.unit.height as f32 / 2.0) * 170.0);
        let (w, h) = if self.rot % 2 == 1 { (u.unit.height, u.unit.width) } else { (u.unit.width, u.unit.height) };
        GroupUnit {
            name: u.unit.unit_folder_name.clone(),
            game: u.unit.game.clone(),
            x,
            z,
            w,
            h,
            rot: (u.unit.rotation + 4 - self.rot) % 4,
        }
    }
}

/// A short name for an object, used as its key in a spot.
fn object_id(so: &SpawnObject) -> Option<String> {
    Some(match so {
        SpawnObject::Teki(t, _) => match &t.carrying {
            Some(c) => format!("t:{}|{c}", t.internal_name),
            None => format!("t:{}", t.internal_name),
        },
        SpawnObject::CapTeki(t, _) => match &t.carrying {
            Some(c) => format!("t:{}|{c}", t.internal_name),
            None => format!("t:{}", t.internal_name),
        },
        SpawnObject::Item(i) => format!("i:{}", i.internal_name),
        SpawnObject::Gate(..) => "g".into(),
        SpawnObject::Hole(_) => "h".into(),
        SpawnObject::Geyser(_) => "y".into(),
        SpawnObject::Ship => "s".into(),
        SpawnObject::Onion(_) => return None,
    })
}

impl Aggregator {
    pub fn new(sublevel: String, grouping: Grouping, comedown_cells: f32) -> Aggregator {
        Aggregator { sublevel, grouping, half: comedown_cells * 85.0, total: 0, dropped: 0, groups: HashMap::new() }
    }

    pub fn add(&mut self, seed: u32, layout: &Layout) {
        self.total += 1;
        let ship = layout.ship.map(|i| &layout.spawnpoints[i as usize]);
        let anchor = match (self.grouping.is_comedown(), ship) {
            (true, Some(sp)) => sp.unit as usize,
            (true, None) => {
                self.dropped += 1;
                return;
            }
            _ => 0,
        };
        let frame = Frame::new(layout, anchor);

        // Every object with the spot it spawned from: spawn points, then seams.
        let mut placed: Vec<(String, Point, Point)> = Vec::new(); // id, spot, actual position
        for sp in layout.spawnpoints.iter() {
            for so in sp.contains.iter() {
                let Some(id) = object_id(so) else { continue };
                let pos = match so {
                    SpawnObject::Teki(_, offset) => sp.pos + *offset,
                    _ => sp.pos,
                };
                placed.push((id, sp.pos, pos));
            }
        }
        for (i, door) in layout.doors.iter().enumerate() {
            let first = door.adjacent.is_none_or(|adj| i < adj as usize);
            if let Some(so) = door.seam.as_ref().filter(|_| first)
                && let Some(id) = object_id(so)
            {
                placed.push((id, door.center(), door.center()));
            }
        }
        let spot = |p: Point| {
            let (x, z) = frame.apply(p[0], p[2]);
            (x.round() as i32, z.round() as i32)
        };

        // The units (and objects) that define the group.
        let (unit_ids, pod): (Vec<usize>, Option<(f32, f32)>) = match self.grouping {
            Grouping::Layout => ((0..layout.units.len()).collect(), None),
            Grouping::Rooms => {
                (layout.units.iter().enumerate().filter(|(_, u)| u.unit.room_type == RoomType::Room).map(|(i, _)| i).collect(), None)
            }
            Grouping::First => (vec![0], None),
            Grouping::ComedownUnits | Grouping::Comedown => {
                let sp = ship.expect("checked above");
                let (sx, sz) = (sp.pos[0], sp.pos[2]);
                let ids = layout
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| {
                        let (x0, z0) = (u.x as f32 * 170.0, u.z as f32 * 170.0);
                        let (x1, z1) = (x0 + u.unit.width as f32 * 170.0, z0 + u.unit.height as f32 * 170.0);
                        x0 < sx + self.half && x1 > sx - self.half && z0 < sz + self.half && z1 > sz - self.half
                    })
                    .map(|(i, _)| i)
                    .collect();
                (ids, Some(frame.apply(sx, sz)))
            }
        };
        let units: Vec<GroupUnit> = unit_ids.iter().map(|&i| frame.unit(layout, i)).collect();
        let mut key_parts: Vec<String> =
            units.iter().map(|u| format!("{} {} {} {}", u.name, u.rot, u.x.round() as i32, u.z.round() as i32)).collect();
        if self.grouping == Grouping::Comedown {
            let sp = ship.expect("checked above");
            for (id, at, pos) in placed.iter() {
                if (pos[0] - sp.pos[0]).abs() <= self.half && (pos[2] - sp.pos[2]).abs() <= self.half && id != "s" {
                    let (x, z) = spot(*at);
                    key_parts.push(format!("{id}@{x},{z}"));
                }
            }
        }
        key_parts.sort();
        let key = key_parts.join("|");

        if !self.groups.contains_key(&key) && self.groups.len() >= MAX_GROUPS {
            self.dropped += 1;
            return;
        }
        let group = self.groups.entry(key).or_default();
        if group.count == 0 {
            group.units = units;
            group.pod = pod;
        }
        group.count += 1;
        if group.seeds.len() < EXAMPLE_SEEDS {
            group.seeds.push(seed);
        }
        for (id, at, _) in placed {
            *group.spots.entry(spot(at)).or_default().entry(id).or_default() += 1;
        }
        // Which cells are covered, to sketch the rest of the cave when the group is only part
        // of the layout. A whole-layout group has all its units already.
        if self.grouping != Grouping::Layout {
            for u in layout.units.iter() {
                for dx in 0..u.unit.width {
                    for dz in 0..u.unit.height {
                        let (x, z) = frame.apply((u.x as f32 + dx as f32 + 0.5) * 170.0, (u.z as f32 + dz as f32 + 0.5) * 170.0);
                        *group.cells.entry(((x / 85.0).round() as i32, (z / 85.0).round() as i32)).or_default() += 1;
                    }
                }
            }
        }
    }

    /// The `max` most common groups as JSON.
    pub fn result_json(&self, max: usize) -> String {
        let mut groups: Vec<&Group> = self.groups.values().collect();
        groups.sort_by(|a, b| b.count.cmp(&a.count).then(a.seeds.first().cmp(&b.seeds.first())));
        let q = crate::q;
        let f = crate::f;
        let out: Vec<String> = groups
            .iter()
            .take(max)
            .map(|g| {
                let units: Vec<String> = g
                    .units
                    .iter()
                    .map(|u| {
                        format!(
                            "{{\"name\":{},\"game\":{},\"x\":{},\"z\":{},\"w\":{},\"h\":{},\"rot\":{}}}",
                            q(&u.name),
                            q(&u.game),
                            f(u.x),
                            f(u.z),
                            u.w,
                            u.h,
                            u.rot
                        )
                    })
                    .collect();
                let mut spots: Vec<(&(i32, i32), &HashMap<String, u64>)> = g.spots.iter().collect();
                spots.sort_by_key(|(k, _)| **k);
                let spots: Vec<String> = spots
                    .iter()
                    .map(|((x, z), objs)| {
                        let mut objs: Vec<(&String, &u64)> = objs.iter().collect();
                        objs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
                        let objs: Vec<String> = objs.iter().map(|(id, n)| format!("[{},{n}]", q(id))).collect();
                        format!("[{x},{z},[{}]]", objs.join(","))
                    })
                    .collect();
                let mut cells: Vec<(&(i32, i32), &u64)> = g.cells.iter().collect();
                cells.sort();
                let cells: Vec<String> = cells.iter().map(|((x, z), n)| format!("[{x},{z},{n}]")).collect();
                let seeds: Vec<String> = g.seeds.iter().map(|s| s.to_string()).collect();
                format!(
                    "{{\"count\":{},\"seeds\":[{}],\"pod\":{},\"units\":[{}],\"spots\":[{}],\"cells\":[{}]}}",
                    g.count,
                    seeds.join(","),
                    g.pod.map_or("null".into(), |(x, z)| format!("[{},{}]", f(x), f(z))),
                    units.join(","),
                    spots.join(","),
                    cells.join(",")
                )
            })
            .collect();
        format!(
            "{{\"total\":{},\"groups_total\":{},\"dropped\":{},\"half\":{},\"groups\":[{}]}}",
            self.total,
            self.groups.len(),
            self.dropped,
            crate::f(self.half),
            out.join(",")
        )
    }
}
