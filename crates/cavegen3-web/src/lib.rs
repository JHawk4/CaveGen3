//! CaveGen3 for the browser.
//!
//! This exposes a small C-style interface so the page needs no JavaScript glue generator:
//!
//! 1. `alloc(len)` reserves `len` bytes; JavaScript writes into them and passes the pointer.
//! 2. `add_files(ptr, len)` adds a bundle of data files (see [`parse_bundle`]). Call it once with
//!    the shared resources and once per game before generating that game's sublevels.
//! 3. `caves()`, `generate(ptr, len, seed, flags)` and `command(ptr, len)` return a pointer to
//!    JSON text, whose length is `result_len()`. The text stays valid until the next call.
//!    `command` takes tab-separated text for the page's advanced options: a judge score, a seed
//!    search, or a sublevel's CaveInfo (see [`Web::command`]).
//!
//! The same functions are plain Rust underneath (see [`Web`]), so they're tested natively.

mod aggregate;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, RwLock},
};

use cavegen3::{
    GameData, Layout, Placement, SpawnObject,
    caveinfo::CaveInfo,
    data::FileSource,
    jhawk_judge::{JhawkJudge, JudgeType},
    point::Point,
    query::{Query, StructuralQuery},
    rng::Rng,
    waypoint::get_path_to_goal,
};

/// `generate` flag: add spawn points, doors, scores, waypoints and carry paths.
pub const DETAILS: u32 = 1;

/// Data files held in memory, shared with [`GameData`] so more games can be added later.
/// Paths match ignoring case, as on the game's file system.
#[derive(Default)]
struct MemSource {
    files: RwLock<BTreeMap<String, Vec<u8>>>,
}

impl MemSource {
    fn insert(&self, entries: Vec<(String, Vec<u8>)>) {
        let mut files = self.files.write().unwrap();
        for (path, data) in entries {
            files.insert(path.to_ascii_lowercase(), data);
        }
    }
}

impl FileSource for &'static MemSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.files.read().unwrap().get(&path.to_ascii_lowercase()).cloned()
    }

    fn list(&self, dir: &str) -> Vec<(String, bool)> {
        let prefix = format!("{}/", dir.trim_end_matches('/').to_ascii_lowercase());
        let files = self.files.read().unwrap();
        let mut out = BTreeSet::new();
        for path in files.keys().filter(|p| p.starts_with(&prefix)) {
            let rest = &path[prefix.len()..];
            match rest.split_once('/') {
                Some((sub, _)) => out.insert((sub.to_string(), true)),
                None => out.insert((rest.to_string(), false)),
            };
        }
        out.into_iter().collect()
    }
}

/// Splits a bundle into `(path, contents)` pairs. A bundle is a sequence of entries, each a
/// little-endian u32 path length, the UTF-8 path, a u32 contents length, then the contents.
pub fn parse_bundle(mut bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    fn take<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], String> {
        if bytes.len() < n {
            return Err("truncated bundle".into());
        }
        let (head, rest) = bytes.split_at(n);
        *bytes = rest;
        Ok(head)
    }
    fn take_u32(bytes: &mut &[u8]) -> Result<usize, String> {
        Ok(u32::from_le_bytes(take(bytes, 4)?.try_into().unwrap()) as usize)
    }
    let mut out = Vec::new();
    while !bytes.is_empty() {
        let n = take_u32(&mut bytes)?;
        let path = String::from_utf8(take(&mut bytes, n)?.to_vec()).map_err(|_| "bundle path isn't UTF-8")?;
        let n = take_u32(&mut bytes)?;
        out.push((path, take(&mut bytes, n)?.to_vec()));
    }
    Ok(out)
}

/// The generator state behind the exported functions.
pub struct Web {
    source: &'static MemSource,
    data: Option<GameData>,
    /// The aggregation in progress (see `agg_start`).
    agg: Option<aggregate::Aggregator>,
}

impl Default for Web {
    fn default() -> Self {
        Web { source: Box::leak(Box::default()), data: None, agg: None }
    }
}

impl Web {
    /// A generator over data already loaded, such as [`GameData::from_dir`] (for native tools).
    pub fn from_data(data: GameData) -> Web {
        Web { data: Some(data), ..Web::default() }
    }

    /// Adds files from a bundle. Parsed caveinfo is cached per [`GameData`], so this starts a
    /// fresh one; that only costs re-reading the cave list.
    pub fn add_files(&mut self, bundle: &[u8]) -> Result<(), String> {
        self.source.insert(parse_bundle(bundle)?);
        self.data = Some(GameData::from_source(Box::new(self.source)).map_err(|e| e.to_string())?);
        Ok(())
    }

    fn data(&self) -> Result<&GameData, String> {
        self.data.as_ref().ok_or_else(|| "no data loaded".to_string())
    }

    /// Every cave as JSON: `[{game, name, challenge, short, aliases, file}]`.
    pub fn caves_json(&self) -> Result<String, String> {
        let data = self.data()?;
        let mut out = String::from("[");
        for (i, c) in data.caves().iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"game\":{},\"name\":{},\"challenge\":{},\"short\":{},\"aliases\":[{}],\"file\":{}}}",
                q(&c.game),
                q(&c.full_name),
                c.is_challenge_mode,
                q(c.short_name()),
                c.shortened_names.iter().map(|s| q(s)).collect::<Vec<_>>().join(","),
                q(&c.caveinfo_filename),
            ));
        }
        out.push(']');
        Ok(out)
    }

    /// Generates one layout and describes it as JSON (see the page's `render` for the fields).
    /// With [`DETAILS`] in `flags` it also has spawn points, doors, scores, waypoints and paths.
    pub fn generate_json(&self, sublevel: &str, seed: u32, flags: u32) -> Result<String, String> {
        let data = self.data()?;
        let sublevel = data.sublevel(sublevel).map_err(|e| e.to_string())?;
        let caveinfo = data.caveinfo(&sublevel).map_err(|e| e.to_string())?;
        let num_floors = data.floors(&sublevel.cfg).map_err(|e| e.to_string())?.len();
        let layout = Layout::generate(seed, caveinfo);
        Ok(layout_json(&layout, &sublevel, num_floors, data, flags & DETAILS != 0))
    }

    /// Runs one tab-separated command and returns JSON:
    ///
    /// - `judge <sublevel> <type> <seed>`: `{"score": n}`
    /// - `search <sublevel> <type or -> <query or -> <start> <count> <c|g|r> <keep>`: tests `count`
    ///   seeds from `start` (see [`seed_at`] for the modes).
    ///   Seeds must match the query; with a judge type the `keep` lowest scores are returned,
    ///   otherwise the first `keep` matches. `{"tested", "matched", "best": [[seed, score]],
    ///   "scores": [...]}`, where `scores` has every matching seed's score, for ranking.
    /// - `caveinfo <sublevel>`: the sublevel's generation parameters (see [`caveinfo_json`]).
    /// - `agg_start <sublevel> <grouping> <near> <far> <near half> <far half> [image periods]`
    ///   (the landing view, see [`aggregate::View`]; image periods are "unit=period;..."), then `agg_run <start> <count> <c|g|r>`
    ///   any number of times, then `agg_result <max groups>`: CaveGen's Aggregator (see
    ///   [`aggregate`]).
    pub fn command(&mut self, text: &str) -> Result<String, String> {
        let args: Vec<&str> = text.split('\t').collect();
        if let Some(out) = self.agg_command(&args) {
            return out;
        }
        self.query(text)
    }

    /// The commands that don't change state (`judge`, `search`, `caveinfo`), callable from
    /// several threads at once (the command-line tool searches in parallel).
    pub fn query(&self, text: &str) -> Result<String, String> {
        let args: Vec<&str> = text.split('\t').collect();
        let data = self.data()?;
        let arg = |i: usize| args.get(i).copied().ok_or_else(|| format!("{}: missing argument {i}", args[0]));
        let num = |i: usize| -> Result<u64, String> { arg(i)?.trim().parse().map_err(|_| format!("bad number {:?}", args[i])) };
        let caveinfo = |i: usize| -> Result<(cavegen3::Sublevel, &CaveInfo), String> {
            let sublevel = data.sublevel(arg(i)?).map_err(|e| e.to_string())?;
            let caveinfo = data.caveinfo(&sublevel).map_err(|e| e.to_string())?;
            Ok((sublevel, caveinfo))
        };
        match args[0] {
            "judge" => {
                let (_, ci) = caveinfo(1)?;
                let judge = JhawkJudge::new(arg(2)?.parse::<JudgeType>()?, ci);
                let layout = Layout::generate(num(3)? as u32, ci);
                Ok(format!("{{\"score\":{}}}", fnum(judge.judge(&layout))))
            }
            "search" => {
                let (sublevel, ci) = caveinfo(1)?;
                let judge = match arg(2)?.trim() {
                    "" | "-" => None,
                    t => Some(JhawkJudge::new(t.parse::<JudgeType>()?, ci)),
                };
                let query = match arg(3)?.trim() {
                    "" | "-" => None,
                    q => Some(parse_query(q, &sublevel, data)?),
                };
                let (start, count, mode, keep) = (num(4)?, num(5)?, arg(6)?, num(7)? as usize);
                // Optional: tally how many of something every tested seed has (CaveGen's -count).
                let tally_what = args.get(8).map(|w| w.trim()).filter(|w| !w.is_empty() && *w != "-");
                let mut tally: BTreeMap<u32, u64> = BTreeMap::new();
                let mut best: Vec<(u32, f64)> = Vec::new();
                let mut scores = Vec::new();
                for i in 0..count {
                    let seed = seed_at(mode, start, i);
                    if let Some(what) = tally_what {
                        *tally.entry(Layout::generate(seed, ci).count(what)).or_default() += 1;
                    }
                    if let Some(q) = &query
                        && !q.matches(seed, data)
                    {
                        continue;
                    }
                    let score = match &judge {
                        Some(j) => j.judge(&Layout::generate(seed, ci)),
                        None => 0.0,
                    };
                    scores.push(fnum(score));
                    if judge.is_some() {
                        let at = best.partition_point(|&(_, s)| s <= score);
                        if at < keep {
                            best.insert(at, (seed, score));
                            best.truncate(keep);
                        }
                    } else if best.len() < keep {
                        best.push((seed, score));
                    }
                }
                let best: Vec<String> = best.iter().map(|(seed, s)| format!("[{seed},{}]", fnum(*s))).collect();
                let tally: Vec<String> = tally.iter().map(|(k, n)| format!("[{k},{n}]")).collect();
                Ok(format!(
                    "{{\"tested\":{count},\"matched\":{},\"best\":[{}],\"scores\":[{}],\"counts\":[{}]}}",
                    scores.len(),
                    best.join(","),
                    if judge.is_some() { scores.join(",") } else { String::new() },
                    tally.join(",")
                ))
            }
            "caveinfo" => Ok(caveinfo_json(caveinfo(1)?.1, data)),
            other => Err(format!("unknown command {other:?}")),
        }
    }
}

impl Web {
    fn agg_command(&mut self, args: &[&str]) -> Option<Result<String, String>> {
        let arg = |i: usize| args.get(i).copied().ok_or_else(|| format!("{}: missing argument {i}", args[0]));
        let mut run = || -> Result<String, String> {
            Ok(match args[0] {
                "agg_start" => {
                    let data = self.data()?;
                    let sublevel = data.sublevel(arg(1)?).map_err(|e| e.to_string())?;
                    data.caveinfo(&sublevel).map_err(|e| e.to_string())?;
                    let nums: Vec<f32> = (3..7)
                        .map(|i| arg(i)?.trim().parse::<f32>().map_err(|_| "bad landing view".to_string()))
                        .collect::<Result<_, _>>()?;
                    let view = aggregate::View { near: nums[0], far: nums[1], near_half: nums[2], far_half: nums[3] };
                    let grouping = aggregate::Grouping::parse(arg(2)?)?;
                    let image_periods = args.get(7).copied().unwrap_or("");
                    self.agg = Some(aggregate::Aggregator::new(arg(1)?.to_string(), grouping, view, image_periods));
                    "{}".into()
                }
                "agg_run" => {
                    let data = self.data.as_ref().ok_or("no data loaded")?;
                    let agg = self.agg.as_mut().ok_or("agg_run before agg_start")?;
                    let sublevel = data.sublevel(&agg.sublevel).map_err(|e| e.to_string())?;
                    let ci = data.caveinfo(&sublevel).map_err(|e| e.to_string())?;
                    let start: u64 = arg(1)?.trim().parse().map_err(|_| "bad start".to_string())?;
                    let count: u64 = arg(2)?.trim().parse().map_err(|_| "bad count".to_string())?;
                    let mode = arg(3)?;
                    for i in 0..count {
                        let seed = seed_at(mode, start, i);
                        agg.add(seed, &Layout::generate(seed, ci));
                    }
                    "{}".into()
                }
                "agg_result" => {
                    let max: usize = arg(1)?.trim().parse().map_err(|_| "bad count".to_string())?;
                    self.agg.as_ref().ok_or("agg_result before agg_start")?.result_json(max)
                }
                _ => return Err(String::new()),
            })
        };
        match run() {
            Err(e) if e.is_empty() => None,
            r => Some(r),
        }
    }
}

/// Parses a seed search query. Queries name their sublevel ("sh6 hole carry dist < 1000"); one
/// that doesn't is about `sublevel`. Names without a `game:` prefix belong to `sublevel`'s game.
fn parse_query(text: &str, sublevel: &cavegen3::Sublevel, data: &GameData) -> Result<StructuralQuery, String> {
    let game = Some(sublevel.cfg.game.as_str());
    match StructuralQuery::try_parse_in(text, data, game) {
        Ok(q) => Ok(q),
        // Only retry when it failed for want of a sublevel at the start.
        Err(e) if e.to_string().contains("1:1") => {
            StructuralQuery::try_parse_in(&format!("{} {text}", sublevel.short_name()), data, game).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// The `i`th seed of a search or aggregation: `start + i` ("c"), the `i`th state of the game's
/// RNG after `start` ("g", CaveGen's -consecutiveSeeds), or a spread-out seed numbered
/// `start + i` ("r", for random seeds).
pub fn seed_at(mode: &str, start: u64, i: u64) -> u32 {
    match mode {
        "g" => {
            let mut rng = Rng::new(start as u32);
            rng.advance(i as u32);
            rng.seed()
        }
        "r" => mix32(start.wrapping_add(i)),
        _ => start.wrapping_add(i) as u32,
    }
}

/// Spreads the numbers 0, 1, 2, ... over all seeds. It's a bijection, so no seed repeats.
fn mix32(i: u64) -> u32 {
    let mut x = (i as u32).wrapping_add(0x9E37_79B9);
    x = (x ^ (x >> 16)).wrapping_mul(0x85EB_CA6B);
    x = (x ^ (x >> 13)).wrapping_mul(0xC2B2_AE35);
    x ^ (x >> 16)
}

/// A score as JSON, rounded to 2 decimals.
fn fnum(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.is_finite() { format!("{r}") } else { "null".into() }
}

/// A sublevel's generation parameters: `{floor, teki, items, gates, caps, units}`.
fn caveinfo_json(ci: &CaveInfo, data: &GameData) -> String {
    let opt = |s: &Option<String>| s.as_deref().map_or("null".into(), q);
    let teki: Vec<String> = ci
        .teki_info
        .iter()
        .map(|t| {
            format!(
                "{{\"name\":{},\"game\":{},\"min\":{},\"weight\":{},\"group\":{},\"carrying\":{},\"falling\":{}}}",
                q(&t.internal_name),
                q(&t.game),
                t.minimum_amount,
                t.filler_distribution_weight,
                t.group,
                opt(&t.carrying),
                t.spawn_method.is_some()
            )
        })
        .collect();
    let items: Vec<String> = ci
        .item_info
        .iter()
        .map(|t| {
            let (value, min_carry, max_carry) = data.treasure(&t.game, &t.internal_name).map_or((0, 0, 0), |t| (t.value, t.min_carry, t.max_carry));
            format!(
                "{{\"name\":{},\"game\":{},\"min\":{},\"weight\":{},\"value\":{value},\"carry\":[{min_carry},{max_carry}]}}",
                q(&t.internal_name),
                q(&t.game),
                t.min_amount,
                t.filler_distribution_weight
            )
        })
        .collect();
    let gates: Vec<String> =
        ci.gate_info.iter().map(|g| format!("{{\"health\":{},\"weight\":{}}}", f(g.health), g.spawn_distribution_weight)).collect();
    let caps: Vec<String> = ci
        .cap_info
        .iter()
        .map(|t| {
            format!(
                "{{\"name\":{},\"game\":{},\"min\":{},\"weight\":{},\"group\":{},\"carrying\":{},\"falling\":{}}}",
                q(&t.internal_name),
                q(&t.game),
                t.minimum_amount,
                t.filler_distribution_weight,
                t.group,
                opt(&t.carrying),
                t.is_falling()
            )
        })
        .collect();
    let mut seen = BTreeSet::new();
    let units: Vec<String> = ci
        .cave_units
        .iter()
        .filter(|u| u.rotation == 0 && seen.insert(u.unit_folder_name.to_ascii_lowercase()))
        .map(|u| {
            format!(
                "{{\"name\":{},\"game\":{},\"w\":{},\"h\":{},\"type\":{},\"doors\":{}}}",
                q(&u.unit_folder_name),
                q(&u.game),
                u.width,
                u.height,
                q(&u.room_type.to_string()),
                u.num_doors
            )
        })
        .collect();
    format!(
        "{{\"floor\":{{\"max_teki\":{},\"max_treasures\":{},\"max_gates\":{},\"rooms\":{},\"corridor_probability\":{},\"cap_probability\":{},\"geyser\":{},\"plugged\":{},\"final\":{},\"waterwraith\":{}}},\"teki\":[{}],\"items\":[{}],\"gates\":[{}],\"caps\":[{}],\"units\":[{}]}}",
        ci.max_main_objects,
        ci.max_treasures,
        ci.max_gates,
        ci.num_rooms,
        f(ci.corridor_probability),
        f(ci.cap_probability),
        ci.has_geyser,
        ci.exit_plugged,
        ci.is_final_floor,
        f(ci.waterwraith_timer),
        teki.join(","),
        items.join(","),
        gates.join(","),
        caps.join(","),
        units.join(","),
    )
}

/// A JSON string literal.
fn q(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A float as JSON, rounded to 2 decimals.
fn f(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.is_finite() { format!("{r}") } else { "0".into() }
}

fn layout_json(layout: &Layout, sublevel: &cavegen3::Sublevel, num_floors: usize, data: &GameData, details: bool) -> String {
    let ci = layout.caveinfo;
    let (mut width, mut height) = (0, 0);
    let mut units = Vec::new();
    for u in layout.units.iter() {
        width = width.max(u.x + u.unit.width as i32);
        height = height.max(u.z + u.unit.height as i32);
        let waterboxes: Vec<String> = u
            .unit
            .waterboxes
            .iter()
            .map(|w| format!("[{},{},{},{}]", f(w.p1[0]), f(w.p1[2]), f(w.p2[0]), f(w.p2[2])))
            .collect();
        let scores = if details { format!(",\"score\":{},\"teki_score\":{}", u.total_score, u.teki_score) } else { String::new() };
        units.push(format!(
            "{{\"name\":{},\"game\":{},\"x\":{},\"z\":{},\"w\":{},\"h\":{},\"rot\":{},\"type\":{},\"water\":[{}]{scores}}}",
            q(&u.unit.unit_folder_name),
            q(&u.unit.game),
            u.x,
            u.z,
            u.unit.width,
            u.unit.height,
            u.unit.rotation,
            q(&u.unit.room_type.to_string()),
            waterboxes.join(","),
        ));
    }

    // Every object with its position and facing, in the same order as Layout::spawn_objects.
    let mut placed: Vec<(&SpawnObject, Point, f32, Placement)> = Vec::new();
    for (si, sp) in layout.spawnpoints.iter().enumerate() {
        for (slot, so) in sp.contains.iter().enumerate() {
            let pos = match so {
                SpawnObject::Teki(_, offset) => sp.pos + *offset,
                _ => sp.pos,
            };
            placed.push((so, pos, sp.angle, Placement::SpawnPoint { index: si as u32, slot: slot as u32 }));
        }
    }
    for (i, door) in layout.doors.iter().enumerate() {
        let first = door.adjacent.is_none_or(|adj| i < adj as usize);
        if let Some(so) = door.seam.as_ref().filter(|_| first) {
            placed.push((so, door.center(), 0.0, Placement::Seam { door: i as u32 }));
        }
    }
    // When each object was placed (CaveGen's -drawSpawnOrder), counting after the map units.
    let order_of = |p: Placement| layout.spawn_order.iter().position(|&o| o == p);

    // Carry paths back to the ship, as CaveGen draws them.
    let graph = (details && layout.ship.is_some()).then(|| layout.waypoint_graph());
    let path_json = |pos: Point| -> String {
        let Some(g) = graph else { return String::new() };
        let path = get_path_to_goal(pos, 1.0, 10001, g.carry_path_wps_nodes(pos).collect());
        // The path has a point per step; every 8th is plenty to draw it.
        let mut pts: Vec<String> = path.iter().step_by(8).map(|p| format!("{},{}", f(p[0]), f(p[2]))).collect();
        if let Some(last) = path.last() {
            pts.push(format!("{},{}", f(last[0]), f(last[2])));
        }
        // CaveGen's -drawPathDists: the path's length in tens of units, at most 999.
        format!(",\"path\":[{}],\"path_dist\":{}", pts.join(","), (path.len() / 10).min(999))
    };

    let mut objects = Vec::new();
    for (so, pos, angle, placement) in placed {
        let (x, z) = (f(pos[0]), f(pos[2]));
        // The pod's facing is always included: the page's landing and comedown views need it.
        let ship_angle = format!(",\"angle\":{}", f(angle));
        let angle = if details { ship_angle.clone() } else { String::new() };
        let obj = match so {
            SpawnObject::Teki(info, _) => format!(
                "{{\"kind\":\"teki\",\"name\":{},\"game\":{},\"group\":{},\"falling\":{},\"carrying\":{},\"x\":{x},\"z\":{z}{angle}{}}}",
                q(&info.internal_name),
                q(&info.game),
                info.group,
                info.spawn_method.is_some(),
                info.carrying.as_deref().map_or("null".into(), q),
                if info.carrying.is_some() { path_json(pos) } else { String::new() },
            ),
            SpawnObject::CapTeki(info, count) => format!(
                "{{\"kind\":\"teki\",\"name\":{},\"game\":{},\"group\":\"cap\",\"falling\":{},\"carrying\":{},\"count\":{count},\"x\":{x},\"z\":{z}{angle}{}}}",
                q(&info.internal_name),
                q(&info.game),
                info.is_falling(),
                info.carrying.as_deref().map_or("null".into(), q),
                if info.carrying.is_some() { path_json(pos) } else { String::new() },
            ),
            SpawnObject::Item(info) => {
                let t = data.treasure(&info.game, &info.internal_name);
                let value = t.map_or(0, |t| t.value);
                let radius = f(t.map_or(0.0, |t| t.radius));
                let buried = t.is_some_and(|t| t.fully_buried());
                format!(
                    "{{\"kind\":\"treasure\",\"name\":{},\"game\":{},\"value\":{value},\"radius\":{radius},\"buried\":{buried},\"x\":{x},\"z\":{z}{angle}{}}}",
                    q(&info.internal_name),
                    q(&info.game),
                    path_json(pos),
                )
            }
            SpawnObject::Gate(info, dir) => {
                format!("{{\"kind\":\"gate\",\"health\":{},\"dir\":{dir},\"x\":{x},\"z\":{z}}}", f(info.health))
            }
            SpawnObject::Hole(plugged) => format!("{{\"kind\":\"hole\",\"plugged\":{plugged},\"x\":{x},\"z\":{z}{angle}}}"),
            SpawnObject::Geyser(_) => format!("{{\"kind\":\"geyser\",\"x\":{x},\"z\":{z}{angle}}}"),
            SpawnObject::Ship => format!("{{\"kind\":\"ship\",\"x\":{x},\"z\":{z}{ship_angle}}}"),
            SpawnObject::Onion(color) => format!("{{\"kind\":\"onion\",\"color\":{color},\"x\":{x},\"z\":{z}{angle}}}"),
        };
        let mut obj = obj;
        if let Some(n) = order_of(placement).filter(|_| details) {
            obj.pop();
            obj.push_str(&format!(",\"order\":{}}}", layout.units.len() + n));
        }
        objects.push(obj);
    }

    let mut extra = String::new();
    if details {
        let spawnpoints: Vec<String> = layout
            .spawnpoints
            .iter()
            .map(|sp| {
                format!(
                    "{{\"x\":{},\"z\":{},\"group\":{},\"r\":{},\"angle\":{},\"min\":{},\"max\":{},\"unit\":{},\"hole_score\":{},\"treasure_score\":{},\"used\":{}}}",
                    f(sp.pos[0]),
                    f(sp.pos[2]),
                    sp.spawnpoint.group,
                    f(sp.spawnpoint.radius),
                    f(sp.angle),
                    sp.spawnpoint.min_num,
                    sp.spawnpoint.max_num,
                    sp.unit,
                    sp.hole_score,
                    sp.treasure_score,
                    !sp.contains.is_empty(),
                )
            })
            .collect();
        let doors: Vec<String> = layout
            .doors
            .iter()
            .map(|d| {
                let c = d.center();
                let first = layout.units[d.unit as usize].first_door as usize;
                let links: Vec<String> = d.door.door_links.iter().map(|l| (first + l.door_id).to_string()).collect();
                format!(
                    "{{\"x\":{},\"z\":{},\"dir\":{},\"unit\":{},\"adjacent\":{},\"score\":{},\"seam_score\":{},\"links\":[{}]}}",
                    f(c[0]),
                    f(c[2]),
                    d.door.direction,
                    d.unit,
                    d.adjacent.map_or("null".into(), |a| a.to_string()),
                    d.door_score,
                    d.seam_teki_score,
                    links.join(","),
                )
            })
            .collect();
        let waypoints: Vec<String> = graph.map_or_else(Vec::new, |g| {
            g.iter()
                .map(|wp| {
                    format!(
                        "{{\"x\":{},\"z\":{},\"r\":{},\"next\":{}}}",
                        f(wp.pos[0]),
                        f(wp.pos[2]),
                        f(wp.r),
                        g.backlink(wp).map_or("null".into(), |b| b.index().to_string())
                    )
                })
                .collect()
        });
        extra = format!(
            ",\"spawnpoints\":[{}],\"doors\":[{}],\"waypoints\":[{}]",
            spawnpoints.join(","),
            doors.join(","),
            waypoints.join(",")
        );
    }

    // Every treasure the sublevel can have, in caveinfo order (the file's TekiInfo carriers, then
    // ItemInfo, then CapInfo carriers), which is the order the generator places them in.
    let mut order: Vec<String> = Vec::new();
    let carried = ci.teki_info.iter().filter_map(|t| t.carrying.as_ref());
    let items = ci.item_info.iter().map(|i| &i.internal_name);
    let caps = ci.cap_info.iter().filter_map(|c| c.carrying.as_ref());
    for name in carried.chain(items).chain(caps) {
        let name = q(&name.to_ascii_lowercase());
        if !order.contains(&name) {
            order.push(name);
        }
    }
    extra.push_str(&format!(",\"treasure_order\":[{}]", order.join(",")));

    format!(
        "{{\"sublevel\":{},\"long_name\":{},\"game\":{},\"floor\":{},\"floors\":{num_floors},\"seed\":\"{:08X}\",\"width\":{width},\"height\":{height},\"final_floor\":{},\"units\":[{}],\"objects\":[{}]{extra}}}",
        q(&sublevel.short_name()),
        q(&sublevel.long_name()),
        q(&sublevel.cfg.game),
        sublevel.floor,
        layout.seed,
        ci.is_final_floor,
        units.join(","),
        objects.join(","),
    )
}

// The exported interface. WebAssembly runs one thread, so a Mutex is only a formality.

static STATE: Mutex<Option<Web>> = Mutex::new(None);
static RESULT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

fn set_result(r: Result<String, String>) -> *const u8 {
    let text = match r {
        Ok(json) => json,
        Err(e) => format!("{{\"error\":{}}}", q(&e)),
    };
    let mut result = RESULT.lock().unwrap();
    *result = text.into_bytes();
    result.as_ptr()
}

fn with_state<T>(f: impl FnOnce(&mut Web) -> T) -> T {
    let mut state = STATE.lock().unwrap();
    f(state.get_or_insert_with(Web::default))
}

/// Reserves `len` bytes for JavaScript to write into. Free them with `dealloc`.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
/// `ptr` and `len` must come from one call to `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
}

/// The length of the text the last `caves`, `generate`, `command` or `add_files` call returned.
#[unsafe(no_mangle)]
pub extern "C" fn result_len() -> usize {
    RESULT.lock().unwrap().len()
}

/// Adds a bundle of files. Returns JSON: `{}` or `{"error": ...}`.
///
/// # Safety
/// `ptr` must point to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_files(ptr: *const u8, len: usize) -> *const u8 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    set_result(with_state(|w| w.add_files(bytes)).map(|_| "{}".to_string()))
}

#[unsafe(no_mangle)]
pub extern "C" fn caves() -> *const u8 {
    set_result(with_state(|w| w.caves_json()))
}

/// Generates the sublevel named by the UTF-8 text at `ptr` (e.g. "SCx6" or "251:AT1").
///
/// # Safety
/// `ptr` must point to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate(ptr: *const u8, len: usize, seed: u32, flags: u32) -> *const u8 {
    let name = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned();
    set_result(with_state(|w| w.generate_json(&name, seed, flags)))
}

/// Runs a tab-separated command (see [`Web::command`]) given as UTF-8 text at `ptr`.
///
/// # Safety
/// `ptr` must point to `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn command(ptr: *const u8, len: usize) -> *const u8 {
    let text = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned();
    set_result(with_state(|w| w.command(&text)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Builds a bundle from files under the data folder, like `tools/build_site.py` does.
    fn bundle(root: &Path, dirs: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        fn walk(root: &Path, dir: &Path, out: &mut Vec<u8>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else if path.extension().is_some_and(|e| e == "txt") {
                    let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                    let data = std::fs::read(&path).unwrap();
                    out.extend((rel.len() as u32).to_le_bytes());
                    out.extend(rel.as_bytes());
                    out.extend((data.len() as u32).to_le_bytes());
                    out.extend(data);
                }
            }
        }
        for d in dirs {
            walk(root, &root.join(d), &mut out);
        }
        out
    }

    #[test]
    fn generates_from_bundles() {
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"));
        let mut web = Web::default();
        web.add_files(&bundle(root, &["resources"])).unwrap();
        assert!(web.generate_json("SCx6", 0, 0).is_err(), "pikmin2 isn't loaded yet");
        web.add_files(&bundle(root, &["assets/pikmin2"])).unwrap();
        web.add_files(&bundle(root, &["assets/251"])).unwrap();

        // Same layout as generating straight from the data folder.
        let data = GameData::bundled().unwrap();
        let sub = data.sublevel("SCx6").unwrap();
        let expected = Layout::generate(0x1234, data.caveinfo(&sub).unwrap());
        let json = web.generate_json("SCx6", 0x1234, 0).unwrap();
        for u in expected.units.iter() {
            assert!(json.contains(&format!("\"name\":\"{}\",", u.unit.unit_folder_name)));
        }
        assert!(json.contains("\"seed\":\"00001234\""));
        assert!(json.contains("\"kind\":\"ship\""));
        assert!(web.generate_json("251:AT1", 7, 0).unwrap().contains("\"game\":\"251\""));
        assert!(web.caves_json().unwrap().contains("\"name\":\"Colossal Caverns\""));
        assert!(web.generate_json("not a cave", 0, 0).is_err());

        // Details, judge, search and caveinfo.
        let detailed = web.generate_json("SCx6", 0x1234, DETAILS).unwrap();
        for key in ["\"spawnpoints\":[{", "\"doors\":[{", "\"waypoints\":[{", "\"path\":[", "\"angle\":", "\"score\":"] {
            assert!(detailed.contains(key), "missing {key}");
        }
        let judge = JhawkJudge::new(JudgeType::Score, data.caveinfo(&sub).unwrap());
        let score = judge.judge(&expected);
        assert_eq!(web.command("judge\tSCx6\tscore\t4660").unwrap(), format!("{{\"score\":{}}}", fnum(score)));
        let found = web.command("search\tSCx6\tscore\t-\t4660\t20\tc\t3").unwrap();
        assert!(found.starts_with("{\"tested\":20,\"matched\":20,\"best\":[["), "{found}");
        let found = web.command("search\tSCx6\t-\thole carry dist < 1500\t0\t50\tr\t5").unwrap();
        assert!(found.starts_with("{\"tested\":50,"), "{found}");
        assert!(web.command("caveinfo\tSCx6").unwrap().contains("\"teki\":[{\"name\":"));
        assert!(web.command("search\tSCx6\t-\tnot a query ((\t0\t5\tc\t5").is_err());
        // A romhack sublevel's query, with or without naming the sublevel, stays in that romhack
        // (251's SS is Subterranean Sands; vanilla has no SS).
        for q in ["hole carry dist < 1500", "ss2 hole carry dist < 1500"] {
            let found = web.command(&format!("search\t251:SS2\t-\t{q}\t0\t5\tr\t5")).unwrap();
            assert!(found.starts_with("{\"tested\":5,"), "{q}: {found}");
        }

        // The aggregator: every seed lands in exactly one group, and each group's spots count
        // one pod per layout.
        for grouping in ["layout", "rooms", "first", "landing", "comedown", "comedown-units"] {
            web.command(&format!("agg_start\tBK1\t{grouping}\t-90\t1030\t165\t545")).unwrap();
            web.command("agg_run\t0\t150\tc").unwrap();
            web.command("agg_run\t150\t50\tr").unwrap();
            let r = web.command("agg_result\t1000").unwrap();
            assert!(r.starts_with("{\"total\":200,"), "{grouping}: {}", &r[..80]);
            let counts: u64 = r
                .split("{\"count\":")
                .skip(1)
                .map(|s| s.split(',').next().unwrap().parse::<u64>().unwrap())
                .sum();
            assert_eq!(counts, 200, "{grouping}");
            assert_eq!(r.matches("[\"s\",").count() > 0, true, "{grouping}");
        }
        // BK1 has one start room; its landing units-only groups are few.
        web.command("agg_start\tBK1\tfirst\t-90\t1030\t165\t545").unwrap();
        web.command("agg_run\t0\t100\tc").unwrap();
        let first = web.command("agg_result\t1000").unwrap();
        assert!(first.contains("\"groups_total\":"), "{first}");
        assert!(web.command("agg_start\tBK1\tnonsense\t-90\t1030\t165\t545").is_err());

        // Game-RNG stepping: seed i is the RNG state i steps after the start.
        let mut rng = Rng::new(0x1234);
        rng.rand_raw();
        rng.rand_raw();
        assert_eq!(seed_at("g", 0x1234, 2), rng.seed());
        assert_eq!(seed_at("c", 0x1234, 2), 0x1236);
        let found = web.command("search\tSCx6\tscore\t-\t4660\t20\tg\t3").unwrap();
        assert!(found.starts_with("{\"tested\":20,"), "{found}");

        // Every pikmin2 sublevel works with details on (the waypoint graph and paths can panic,
        // which in the browser would stop the page).
        web.add_files(&bundle(root, &["assets/newyear", "assets/colossal", "assets/216", "assets/bikmin"])).unwrap();
        for cave in data.caves().iter() {
            // New Year's Timeless Realm is missing a unit file.
            let Ok(floors) = data.floors(cave) else { continue };
            for floor in 1..=floors.len() {
                let name = format!("{}:{}-{floor}", cave.game, cave.short_name());
                for seed in [0u32, 0x1234_5678, 0xFFFF_FFFF] {
                    web.generate_json(&name, seed, DETAILS).unwrap();
                }
                for judge in JudgeType::ALL {
                    web.command(&format!("judge\t{name}\t{judge}\t7")).unwrap();
                }
            }
        }
    }
}
