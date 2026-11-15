//! CaveGen's Aggregator: generate many seeds of a sublevel, group the layouts that look alike,
//! and count what spawns at each spot in each group.
//!
//! Positions are taken relative to an anchor unit (the first unit placed, or for comedown
//! grouping the unit the pod is in) and turned by quarter turns, so the same arrangement in any
//! rotation lands in one group:
//!
//! - Comedown groups are turned so the pod (and the camera when you land) faces as close to up
//!   as a quarter turn allows.
//! - Other groups try all four turns and keep the one whose description sorts first.
//!
//! Map units that look the same after a half or quarter turn (like a four-way room) are matched
//! in any of those rotations; see [`symmetry_period`].

use std::collections::HashMap;

use cavegen3::{
    Layout, SpawnObject,
    caveinfo::{CaveUnit, RoomType},
    point::Point,
};

/// How layouts are grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    /// The whole layout: every map unit and where it is (CaveGen's default).
    Layout,
    /// Only the rooms (CaveGen's -aggRooms).
    Rooms,
    /// Only the first room placed (CaveGen's -aggFirst).
    First,
    /// What you see when you land: the map units in view of the camera.
    ComedownUnits,
    /// The comedown's map units plus the objects in view.
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

/// The floor in view when you land: a trapezoid ahead of the pod, in game units. `near` and
/// `far` are distances ahead of the pod (`near` is negative: a little behind it), with the
/// half-widths there. The page works these out from the game's camera settings.
#[derive(Debug, Clone, Copy)]
pub struct View {
    pub near: f32,
    pub far: f32,
    pub near_half: f32,
    pub far_half: f32,
}

impl View {
    /// The corners around a pod at `p` facing `f` (a unit vector): near left, near right, far
    /// right, far left.
    fn corners(&self, p: (f32, f32), f: (f32, f32)) -> [(f32, f32); 4] {
        let r = (-f.1, f.0);
        let at = |ahead: f32, side: f32| (p.0 + f.0 * ahead + r.0 * side, p.1 + f.1 * ahead + r.1 * side);
        [at(self.near, -self.near_half), at(self.near, self.near_half), at(self.far, self.far_half), at(self.far, -self.far_half)]
    }

    fn contains(&self, p: (f32, f32), f: (f32, f32), q: (f32, f32)) -> bool {
        let (dx, dz) = (q.0 - p.0, q.1 - p.1);
        let ahead = dx * f.0 + dz * f.1;
        let side = -dx * f.1 + dz * f.0;
        if ahead < self.near || ahead > self.far {
            return false;
        }
        let t = (ahead - self.near) / (self.far - self.near);
        side.abs() <= self.near_half + (self.far_half - self.near_half) * t
    }
}

/// Whether two convex polygons overlap (separating axis test). Touching edges don't count.
fn polygons_overlap(a: &[(f32, f32)], b: &[(f32, f32)]) -> bool {
    for poly in [a, b] {
        for i in 0..poly.len() {
            let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
            let axis = (q.1 - p.1, p.0 - q.0);
            let project = |pts: &[(f32, f32)]| {
                pts.iter().map(|v| v.0 * axis.0 + v.1 * axis.1).fold((f32::MAX, f32::MIN), |(lo, hi), d| (lo.min(d), hi.max(d)))
            };
            let ((alo, ahi), (blo, bhi)) = (project(a), project(b));
            if ahi <= blo || bhi <= alo {
                return false;
            }
        }
    }
    true
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

/// A pod position and the way it faces (a unit vector).
type Pod = ((f32, f32), (f32, f32));

#[derive(Debug, Default)]
struct Group {
    count: u64,
    seeds: Vec<u32>,
    /// The units that define the group (from its first layout).
    units: Vec<GroupUnit>,
    /// The pod, for comedown groups.
    pod: Option<Pod>,
    /// What spawned at each spot: spot -> object -> count.
    spots: HashMap<(i32, i32), HashMap<String, u64>>,
    /// How often each half-cell position was covered by a map unit (keys are doubled cell
    /// coordinates of cell centers).
    cells: HashMap<(i32, i32), u64>,
}

pub struct Aggregator {
    pub sublevel: String,
    grouping: Grouping,
    view: View,
    total: u64,
    dropped: u64,
    groups: HashMap<String, Group>,
    /// Each map unit's symmetry period, by game and name (see [`symmetry_period`]).
    periods: HashMap<(String, String), u16>,
    /// Symmetry periods of the units' radar images, worked out by the page (lowercase names).
    /// A unit counts as symmetric only if its shape and its image both are.
    image_periods: HashMap<String, u16>,
}

/// Moves positions to be relative to the anchor's center, then turns them `rot` quarter turns
/// back (a unit's rotation 1 maps (x, z) to (-z, x); this undoes it).
#[derive(Clone, Copy)]
struct Frame {
    cx: f32,
    cz: f32,
    rot: u16,
}

impl Frame {
    fn new(layout: &Layout, anchor: usize, rot: u16) -> Frame {
        let a = &layout.units[anchor];
        Frame { cx: (a.x as f32 + a.unit.width as f32 / 2.0) * 170.0, cz: (a.z as f32 + a.unit.height as f32 / 2.0) * 170.0, rot }
    }

    fn turn(&self, (mut x, mut z): (f32, f32)) -> (f32, f32) {
        for _ in 0..self.rot {
            (x, z) = (z, -x);
        }
        (x, z)
    }

    fn apply(&self, x: f32, z: f32) -> (f32, f32) {
        self.turn((x - self.cx, z - self.cz))
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
            rot: (u.unit.rotation + 4 - self.rot % 4) % 4,
        }
    }
}

/// The smallest number of quarter turns (1, 2 or 4) after which a map unit looks the same: its
/// size, doors and water all land where the unturned unit's are. A four-way junction or a
/// four-way room like Frontier Cavern's cross room is 1; a straight hallway is 2.
///
/// Spawn points are left out on purpose: they rarely match after a turn (the cross room's
/// don't), but you can't see them, so the turned room still looks the same. Seeds whose room is
/// turned then share a group, and their spawn spots overlay on one map.
pub fn symmetry_period(units: &[CaveUnit], name: &str) -> u16 {
    let copies: Vec<&CaveUnit> = units.iter().filter(|u| u.unit_folder_name == name).collect();
    let Some(base) = copies.iter().find(|u| u.rotation == 0) else { return 4 };
    let r = |v: f32| (v * 2.0).round() as i32;
    let door_set = |u: &CaveUnit| {
        let mut v: Vec<(u16, u16)> = u.doors.iter().map(|d| (d.direction, d.side_lateral_offset)).collect();
        v.sort();
        v
    };
    let water_set = |u: &CaveUnit| {
        let mut v: Vec<(i32, i32, i32, i32)> = u.waterboxes.iter().map(|w| (r(w.p1[0]), r(w.p1[2]), r(w.p2[0]), r(w.p2[2]))).collect();
        v.sort();
        v
    };
    let same = |a: &CaveUnit, b: &CaveUnit| {
        a.width == b.width && a.height == b.height && door_set(a) == door_set(b) && water_set(a) == water_set(b)
    };
    for period in [1u16, 2] {
        if let Some(turned) = copies.iter().find(|u| u.rotation == period)
            && same(base, turned)
        {
            return period;
        }
    }
    4
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
    /// `image_periods` is "name=period;name=period..." from the page's look at each unit's radar
    /// image; units it leaves out are judged by shape alone.
    pub fn new(sublevel: String, grouping: Grouping, view: View, image_periods: &str) -> Aggregator {
        let image_periods = image_periods
            .split(';')
            .filter_map(|kv| kv.split_once('='))
            .filter_map(|(k, v)| Some((k.trim().to_ascii_lowercase(), v.trim().parse().ok()?)))
            .collect();
        Aggregator { sublevel, grouping, view, total: 0, dropped: 0, groups: HashMap::new(), periods: HashMap::new(), image_periods }
    }

    fn period(&mut self, layout: &Layout, unit: &CaveUnit) -> u16 {
        let key = (unit.game.clone(), unit.unit_folder_name.clone());
        let image = self.image_periods.get(&unit.unit_folder_name.to_ascii_lowercase()).copied().unwrap_or(1);
        // Periods are 1, 2 or 4, each dividing the next, so the larger one is what both allow.
        *self.periods.entry(key).or_insert_with(|| symmetry_period(&layout.caveinfo.cave_units, &unit.unit_folder_name).max(image))
    }

    pub fn add(&mut self, seed: u32, layout: &Layout) {
        self.total += 1;
        let ship = layout.ship.map(|i| &layout.spawnpoints[i as usize]);
        if self.grouping.is_comedown() && ship.is_none() {
            self.dropped += 1;
            return;
        }

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

        // The units (and, for comedown, objects) that define the group.
        let pod: Option<Pod> = ship.map(|sp| {
            let a = sp.angle.to_radians();
            ((sp.pos[0], sp.pos[2]), (a.sin(), a.cos()))
        });
        let (anchor, unit_ids, in_view): (usize, Vec<usize>, Vec<usize>) = match self.grouping {
            Grouping::Layout => (0, (0..layout.units.len()).collect(), vec![]),
            Grouping::Rooms => {
                (0, (0..layout.units.len()).filter(|&i| layout.units[i].unit.room_type == RoomType::Room).collect(), vec![])
            }
            Grouping::First => (0, vec![0], vec![]),
            Grouping::ComedownUnits | Grouping::Comedown => {
                let (p, f) = pod.expect("checked above");
                let view = self.view.corners(p, f);
                let ids = (0..layout.units.len())
                    .filter(|&i| {
                        let u = &layout.units[i];
                        let (x0, z0) = (u.x as f32 * 170.0, u.z as f32 * 170.0);
                        let (x1, z1) = (x0 + u.unit.width as f32 * 170.0, z0 + u.unit.height as f32 * 170.0);
                        polygons_overlap(&view, &[(x0, z0), (x1, z0), (x1, z1), (x0, z1)])
                    })
                    .collect();
                let objs = if self.grouping == Grouping::Comedown {
                    (0..placed.len())
                        .filter(|&i| placed[i].0 != "s" && self.view.contains(p, f, (placed[i].2[0], placed[i].2[2])))
                        .collect()
                } else {
                    vec![]
                };
                (ship.expect("checked above").unit as usize, ids, objs)
            }
        };
        let periods: Vec<u16> = unit_ids.iter().map(|&i| self.period(layout, layout.units[i].unit)).collect();
        let key_for = |frame: &Frame| -> String {
            let mut parts: Vec<String> = unit_ids
                .iter()
                .zip(periods.iter())
                .map(|(&i, &period)| {
                    let u = frame.unit(layout, i);
                    format!("{} {} {} {}", u.name, u.rot % period, u.x.round() as i32, u.z.round() as i32)
                })
                .collect();
            for &i in in_view.iter() {
                let (x, z) = frame.apply(placed[i].1[0], placed[i].1[2]);
                parts.push(format!("{}@{},{}", placed[i].0, x.round() as i32, z.round() as i32));
            }
            parts.sort();
            parts.join("|")
        };
        let frame = if self.grouping.is_comedown() {
            // Turn so the pod faces as close to up (-z) as possible.
            let (_, f) = pod.expect("checked above");
            (0..4u16)
                .map(|rot| Frame::new(layout, anchor, rot))
                .min_by(|a, b| a.turn(f).1.total_cmp(&b.turn(f).1))
                .expect("four turns")
        } else {
            (0..4u16).map(|rot| Frame::new(layout, anchor, rot)).min_by_key(key_for).expect("four turns")
        };
        let key = key_for(&frame);

        if !self.groups.contains_key(&key) && self.groups.len() >= MAX_GROUPS {
            self.dropped += 1;
            return;
        }
        let is_comedown = self.grouping.is_comedown();
        let grouping = self.grouping;
        let group = self.groups.entry(key).or_default();
        if group.count == 0 {
            group.units = unit_ids.iter().map(|&i| frame.unit(layout, i)).collect();
            group.pod = pod.filter(|_| is_comedown).map(|(p, f)| (frame.apply(p.0, p.1), frame.turn(f)));
        }
        group.count += 1;
        if group.seeds.len() < EXAMPLE_SEEDS {
            group.seeds.push(seed);
        }
        let spot = |p: Point| {
            let (x, z) = frame.apply(p[0], p[2]);
            (x.round() as i32, z.round() as i32)
        };
        for (id, at, _) in placed {
            *group.spots.entry(spot(at)).or_default().entry(id).or_default() += 1;
        }
        // Which cells are covered, to sketch the rest of the cave when the group is only part
        // of the layout. A whole-layout group has all its units already.
        if grouping != Grouping::Layout {
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
                    g.pod.map_or("null".into(), |((x, z), (fx, fz))| format!("[{},{},{},{}]", f(x), f(z), f(fx), f(fz))),
                    units.join(","),
                    spots.join(","),
                    cells.join(",")
                )
            })
            .collect();
        format!(
            "{{\"total\":{},\"groups_total\":{},\"dropped\":{},\"view\":[{},{},{},{}],\"groups\":[{}]}}",
            self.total,
            self.groups.len(),
            self.dropped,
            crate::f(self.view.near),
            crate::f(self.view.far),
            crate::f(self.view.near_half),
            crate::f(self.view.far_half),
            out.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cavegen3::GameData;

    use super::*;

    /// Lists every vanilla map unit that looks the same after a quarter or half turn, and checks
    /// a few known ones.
    #[test]
    fn symmetric_units() {
        let data = GameData::bundled().unwrap();
        let mut periods: BTreeMap<String, u16> = BTreeMap::new();
        for cave in data.caves().iter().filter(|c| c.game == "pikmin2") {
            for ci in data.floors(cave).unwrap() {
                for u in ci.cave_units.iter().filter(|u| u.rotation == 0) {
                    periods.entry(u.unit_folder_name.clone()).or_insert_with(|| symmetry_period(&ci.cave_units, &u.unit_folder_name));
                }
            }
        }
        for (name, p) in periods.iter().filter(|(_, p)| **p < 4) {
            println!("{p} {name}");
        }
        // Straight hallways look the same turned around.
        assert_eq!(periods["way2_metal"], 2);
        // An alcove has one door, so it never does.
        assert_eq!(periods["cap_metal"], 4);
        // Frontier Cavern's cross room looks the same every quarter turn.
        assert_eq!(periods["room_cent2_4_tsuchi"], 1);
    }
}


#[cfg(test)]
mod rotation_tests {
    use std::collections::BTreeSet;

    use cavegen3::{GameData, Layout};

    use super::*;

    fn view() -> View {
        View { near: -90.0, far: 1030.0, near_half: 165.0, far_half: 545.0 }
    }

    /// Grouping by the first room puts every rotation of a room in one group, so there are at
    /// most as many groups as rooms (CaveGen's -aggFirst kept each rotation apart only when
    /// the anchor's own rotation differed).
    #[test]
    fn first_room_groups_ignore_rotation() {
        let data = GameData::bundled().unwrap();
        for name in ["SCx1", "FC5", "HoB2", "SH3"] {
            let ci = data.caveinfo(&data.sublevel(name).unwrap()).unwrap();
            let mut agg = Aggregator::new(name.into(), Grouping::First, view(), "");
            let mut rooms = BTreeSet::new();
            let mut rotations = BTreeSet::new();
            for seed in 0..400u32 {
                let layout = Layout::generate(seed.wrapping_mul(2_654_435_761), ci);
                rooms.insert(layout.units[0].unit.unit_folder_name.clone());
                rotations.insert((layout.units[0].unit.unit_folder_name.clone(), layout.units[0].unit.rotation));
                agg.add(seed, &layout);
            }
            assert!(agg.groups.len() <= rooms.len(), "{name}: {} groups for {} rooms", agg.groups.len(), rooms.len());
            assert!(rotations.len() > rooms.len(), "{name}: the test needs rooms in several rotations");
        }
    }

    /// Comedown groups don't depend on how the whole layout is turned: the same landing seen
    /// from the pod is one group.
    #[test]
    fn comedown_groups_are_fewer_than_layouts() {
        let data = GameData::bundled().unwrap();
        let ci = data.caveinfo(&data.sublevel("HoB2").unwrap()).unwrap();
        let mut by_view = Aggregator::new("HoB2".into(), Grouping::ComedownUnits, view(), "");
        let mut by_layout = Aggregator::new("HoB2".into(), Grouping::Layout, view(), "");
        for seed in 0..2000u32 {
            let layout = Layout::generate(seed.wrapping_mul(2_654_435_761), ci);
            by_view.add(seed, &layout);
            by_layout.add(seed, &layout);
        }
        assert!(by_view.groups.len() < by_layout.groups.len());
        assert_eq!(by_view.groups.values().map(|g| g.count).sum::<u64>(), 2000);
    }
}
