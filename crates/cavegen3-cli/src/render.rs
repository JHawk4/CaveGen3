//! Draws a layout as a PNG, the same way as the web page's map (`drawMap` in `web/app.js`).
//!
//! The layout comes from the web crate's JSON, so the image shows exactly what the page does.

use std::{
    collections::HashMap,
    f32::consts::PI,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use tiny_skia::{
    Color, FillRule, FilterQuality, LineCap, LineJoin, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, StrokeDash, Transform,
};

/// Pixels per map grid cell at scale 1.
pub const CELL: f32 = 40.0;
/// Game units per grid cell.
const UNIT: f32 = 170.0;

// ---- The layout, as the web crate describes it ----

#[derive(Deserialize)]
pub struct Layout {
    pub sublevel: String,
    pub long_name: String,
    pub game: String,
    pub seed: String,
    pub width: f32,
    pub height: f32,
    pub units: Vec<Unit>,
    pub objects: Vec<Object>,
    #[serde(default)]
    pub spawnpoints: Vec<Spawnpoint>,
    #[serde(default)]
    pub doors: Vec<Door>,
    #[serde(default)]
    pub waypoints: Vec<Waypoint>,
}

#[derive(Deserialize)]
pub struct Unit {
    pub name: String,
    pub game: String,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub h: f32,
    pub rot: u32,
    pub water: Vec<[f32; 4]>,
    pub score: Option<i64>,
    pub teki_score: Option<i64>,
}

#[derive(Deserialize)]
pub struct Object {
    pub kind: String,
    pub name: Option<String>,
    pub game: Option<String>,
    /// A spawn group number, or "cap" for teki in a cap's spawn point.
    pub group: Option<serde_json::Value>,
    #[serde(default)]
    pub falling: bool,
    pub carrying: Option<String>,
    pub count: Option<u32>,
    pub x: f32,
    pub z: f32,
    pub angle: Option<f32>,
    pub path: Option<Vec<f32>>,
    pub health: Option<f32>,
    pub dir: Option<u32>,
    #[serde(default)]
    pub plugged: bool,
    pub color: Option<u32>,
    /// A treasure's bounding radius, which the treasure gauge subtracts from the distance.
    pub radius: Option<f32>,
    /// A treasure that starts fully under the ground.
    #[serde(default)]
    pub buried: bool,
    /// When it was placed, counting the map units first (with the generator's details).
    pub order: Option<usize>,
    /// How far it's carried to the pod, in tens of units (with the generator's details).
    pub path_dist: Option<u32>,
}

impl Object {
    fn group(&self) -> Option<i64> {
        self.group.as_ref().and_then(|g| g.as_i64())
    }

    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    fn game(&self) -> &str {
        self.game.as_deref().unwrap_or("pikmin2")
    }
}

#[derive(Deserialize)]
pub struct Spawnpoint {
    pub x: f32,
    pub z: f32,
    pub group: usize,
    pub r: f32,
    pub angle: f32,
    pub hole_score: i64,
    pub treasure_score: i64,
    pub used: bool,
}

#[derive(Deserialize)]
pub struct Door {
    pub x: f32,
    pub z: f32,
    pub dir: usize,
    pub unit: i64,
    pub score: i64,
    pub links: Vec<usize>,
}

#[derive(Deserialize)]
pub struct Waypoint {
    pub x: f32,
    pub z: f32,
    pub r: f32,
    pub next: Option<usize>,
}

// ---- Options ----

/// What to draw: the web page's "Show on the map" options, the Water box and the landing view.
#[derive(Clone)]
pub struct Options {
    pub water: bool,
    pub treasures: bool,
    /// Hide treasures that start fully buried (CaveGen's -drawNoBuriedItems).
    pub hide_buried: bool,
    pub teki: bool,
    pub plants: bool,
    pub gates: bool,
    /// Each gate's HP, written on the gate.
    pub gate_health: bool,
    pub exits: bool,
    pub spawnpoints: bool,
    pub angles: bool,
    pub unit_scores: bool,
    pub spawn_scores: bool,
    pub teki_scores: bool,
    pub door_scores: bool,
    pub door_links: bool,
    pub door_ids: bool,
    pub waypoints: bool,
    pub paths: bool,
    pub gauge: bool,
    pub grid: bool,
    /// Outlines of what the camera shows when you land: after pressing L (the way the pod faces),
    /// and without L as Olimar or as Louie.
    pub landing_box: bool,
    pub landing_box_olimar: bool,
    pub landing_box_louie: bool,
    /// An outline of what the camera shows while the sublevel's name is on screen, as the pod comes down.
    pub comedown_box: bool,
    /// Only the objects the game draws while the pod comes down (see [`comedown_visible`]).
    pub comedown_visible: bool,
    /// CaveGen's -drawSpawnOrder: map units and objects numbered in placement order.
    pub spawn_order: bool,
    /// CaveGen's -drawPathDists: carry distances in tens of units.
    pub path_dists: bool,
    /// CaveGen's -drawPodAngle: an arrow for the pod's facing.
    pub pod_angle: bool,
    /// CaveGen's QuickGlance: translucent discs behind what matters.
    pub quick_glance: bool,
    /// CaveGen's Pretty: rendered floor pictures for the vanilla map units.
    pub pretty: bool,
    /// Draw only what one camera shows, turned as on screen.
    pub view: Option<View>,
    /// The screen's width over its height, for the landing and comedown views.
    pub aspect: f32,
    /// 1 draws a grid cell as [`CELL`] pixels.
    pub scale: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            water: true,
            treasures: true,
            hide_buried: false,
            teki: true,
            plants: true,
            gates: true,
            gate_health: true,
            exits: true,
            spawnpoints: false,
            angles: false,
            unit_scores: false,
            spawn_scores: false,
            teki_scores: false,
            door_scores: false,
            door_links: false,
            door_ids: false,
            waypoints: false,
            paths: false,
            gauge: false,
            grid: false,
            landing_box: false,
            landing_box_olimar: false,
            landing_box_louie: false,
            comedown_box: false,
            comedown_visible: false,
            spawn_order: false,
            path_dists: false,
            pod_angle: false,
            quick_glance: false,
            pretty: false,
            view: None,
            aspect: 16.0 / 9.0,
            scale: 1.0,
        }
    }
}

/// The names `Options::set` takes, as the web page calls them.
pub const OPTION_NAMES: &[&str] = &[
    "water",
    "treasures",
    "hideBuried",
    "teki",
    "plants",
    "gates",
    "gateHealth",
    "exits",
    "comedownVisible",
    "spawnpoints",
    "angles",
    "unitScores",
    "spawnScores",
    "tekiScores",
    "doorScores",
    "doorLinks",
    "doorIds",
    "waypoints",
    "paths",
    "gauge",
    "grid",
    "landingBox",
    "landingBoxOlimar",
    "landingBoxLouie",
    "comedownBox",
    "spawnOrder",
    "pathDists",
    "podAngle",
    "quickGlance",
    "pretty",
];

impl Options {
    /// Turns one option on or off. Names ignore case, `-` and `_`: `doorScores`, `door-scores`.
    pub fn set(&mut self, name: &str, on: bool) -> Result<(), String> {
        let key: String = name
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_ascii_lowercase();
        let field = match key.as_str() {
            "water" => &mut self.water,
            "treasures" => &mut self.treasures,
            "hideburied" => &mut self.hide_buried,
            "teki" => &mut self.teki,
            "plants" => &mut self.plants,
            "gates" => &mut self.gates,
            "gatehealth" => &mut self.gate_health,
            "exits" => &mut self.exits,
            "comedownvisible" => &mut self.comedown_visible,
            "spawnpoints" => &mut self.spawnpoints,
            "angles" => &mut self.angles,
            "unitscores" => &mut self.unit_scores,
            "spawnscores" => &mut self.spawn_scores,
            "tekiscores" => &mut self.teki_scores,
            "doorscores" => &mut self.door_scores,
            "doorlinks" => &mut self.door_links,
            "doorids" => &mut self.door_ids,
            "waypoints" => &mut self.waypoints,
            "paths" => &mut self.paths,
            "gauge" => &mut self.gauge,
            "grid" => &mut self.grid,
            "landingbox" => &mut self.landing_box,
            "landingboxolimar" => &mut self.landing_box_olimar,
            "landingboxlouie" => &mut self.landing_box_louie,
            "comedownbox" => &mut self.comedown_box,
            "spawnorder" => &mut self.spawn_order,
            "pathdists" => &mut self.path_dists,
            "podangle" => &mut self.pod_angle,
            "quickglance" => &mut self.quick_glance,
            "pretty" => &mut self.pretty,
            _ => return Err(format!("unknown map option {name:?}; the options are {}", OPTION_NAMES.join(", "))),
        };
        *field = on;
        Ok(())
    }

    /// Whether the layout needs the generator's details (spawn points, doors, scores, paths).
    pub fn needs_details(&self) -> bool {
        self.spawn_order
            || self.path_dists
            || self.spawnpoints
            || self.angles
            || self.unit_scores
            || self.spawn_scores
            || self.teki_scores
            || self.door_scores
            || self.door_links
            || self.door_ids
            || self.waypoints
            || self.paths
    }

    pub fn shows(&self, o: &Object) -> bool {
        if self.comedown_visible && !comedown_visible(&o.kind) {
            return false;
        }
        match o.kind.as_str() {
            "treasure" => self.treasures && !(self.hide_buried && o.buried),
            "teki" if o.group() == Some(6) => self.plants,
            "teki" => self.teki,
            "gate" => self.gates,
            _ => self.exits,
        }
    }
}

// ---- The landing and comedown views ----

/// Whether the game draws objects of this kind ("treasure", "teki", "gate", "hole", "geyser",
/// "ship", "onion") while the pod comes down.
///
/// The cave intro cutscene s0B_cv_coursein has draw flags 5 in `/user/Mukki/movie/demos.txt`:
/// Pikmin (1) and pellets (4), not enemies (2) or captains (8). `MoviePlayer::setPauseAndDraw`
/// passes those to each manager, and with enemies off `GeneralEnemyMgr` draws only the
/// cutscene's own actors, so no teki shows: plants, eggs and Candypop Buds are teki too, and so
/// are treasures a teki carries, which only become pellets when it dies. Treasures are pellets
/// and show (a buried one as its dirt mound); gates, the hole, the geyser and Onions are items,
/// which cutscenes don't hide. Checked in run videos: EC-1's first treasure shows, HoB-4's
/// Red Bulborb beside the pod doesn't.
pub fn comedown_visible(kind: &str) -> bool {
    kind != "teki"
}

/// A camera looking the way the pod faces: `dist` units back from its target, `angle` degrees
/// down, with a vertical field of view of `fov` degrees, aimed `lift` units above the target,
/// which is `look_ahead` units ahead of the pod. See `LANDING_CAMERA` in web/app.js.
#[derive(Clone, Copy)]
pub struct Camera {
    pub dist: f32,
    pub angle: f32,
    pub fov: f32,
    pub lift: f32,
    pub look_ahead: f32,
}

/// The landing view: the game's Far(Low) settings, fitted to run videos.
pub const LANDING_CAMERA: Camera = Camera {
    dist: 900.0,
    angle: 25.0,
    fov: 26.0,
    lift: 40.0,
    look_ahead: -55.0,
};

/// The comedown camera, while the sublevel's name is on screen as the pod comes down: frame 0
/// of the cutscene s0B_cv_coursein (`/user/Mukki/movie/s0B_cv_coursein/demo.szs`), which holds
/// still until the name fades. The cutscene is placed at the pod with its +Z along the pod's
/// facing; the camera looks almost straight down from 1,321 units up, turned by its roll.
pub struct CutsceneCamera {
    /// Eye and target, in the cutscene's frame: x to the pod's left, y up from the floor, z ahead.
    pub eye: [f32; 3],
    pub target: [f32; 3],
    /// Vertical field of view, degrees.
    pub fovy: f32,
    /// Degrees, as JStudio's camera applies it: `RotZ(-roll) * LookAt(eye, +Y, target)`.
    pub roll: f32,
}

pub const COMEDOWN_CAMERA: CutsceneCamera = CutsceneCamera {
    eye: [-0.286, 1320.99, -3.170],
    target: [4.737, 6.729, -1.692],
    fovy: 74.11,
    roll: 87.82,
};

/// Outline colors: white for the pod, the captains' cursor colors, and yellow for the comedown.
pub const LANDING_COLOR: &str = "#f2f2f2";
pub const OLIMAR_COLOR: &str = "#ff5a4d";
pub const LOUIE_COLOR: &str = "#4d9bff";
pub const COMEDOWN_COLOR: &str = "#ffe14d";

/// The floor a camera sees: a trapezoid from `near` to `far` units ahead of the pod.
#[derive(Clone, Copy)]
pub struct Shape {
    pub near: f32,
    pub far: f32,
    pub near_half: f32,
    pub far_half: f32,
}

/// The floor in view, for a screen `aspect` wide over its height.
pub fn view_shape(cam: Camera, aspect: f32) -> Shape {
    let rad = PI / 180.0;
    let height = cam.dist * (cam.angle * rad).sin() + cam.lift;
    let back = cam.dist * (cam.angle * rad).cos() - cam.look_ahead;
    let half = cam.fov / 2.0 * rad;
    let tan_side = half.tan() * aspect;
    // Where the screen's bottom and top edges meet the floor.
    let edge = |pitch: f32| {
        let ground = height / pitch.tan();
        let depth = ground.hypot(height) * half.cos();
        (ground - back, depth * tan_side)
    };
    let (near, near_half) = edge(cam.angle * rad + half);
    let (far, far_half) = edge(cam.angle * rad - half);
    Shape {
        near,
        far,
        near_half,
        far_half,
    }
}

/// What the camera shows when you land.
pub fn landing_shape(aspect: f32) -> Shape {
    view_shape(LANDING_CAMERA, aspect)
}

impl Shape {
    /// The trapezoid's corners as (ahead, side) from the pod; side is to its right.
    pub fn corners(&self) -> [(f32, f32); 4] {
        [
            (self.near, -self.near_half),
            (self.near, self.near_half),
            (self.far, self.far_half),
            (self.far, -self.far_half),
        ]
    }
}

/// The floor a cutscene camera sees: where the rays through the screen's corners meet the floor,
/// as (ahead, side) from the pod; side is to its right.
pub fn cutscene_corners(cam: &CutsceneCamera, aspect: f32) -> [(f32, f32); 4] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let norm = |a: [f32; 3]| {
        let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        [a[0] / l, a[1] / l, a[2] / l]
    };
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    // The view's axes (C_MTXLookAt): back, right and up.
    let back = norm(sub(cam.eye, cam.target));
    let right = norm(cross([0.0, 1.0, 0.0], back));
    let up = cross(back, right);
    let t = (cam.fovy / 2.0).to_radians().tan();
    let (sin, cos) = cam.roll.to_radians().sin_cos();
    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(sx, sy): (f32, f32)| {
        // A screen corner's ray, turned back by the roll into the look-at frame.
        let (x, y) = (sx * t * aspect, sy * t);
        let (x, y) = (x * cos - y * sin, x * sin + y * cos);
        let d: [f32; 3] = std::array::from_fn(|i| right[i] * x + up[i] * y - back[i]);
        let k = -cam.eye[1] / d[1];
        (cam.eye[2] + k * d[2], -(cam.eye[0] + k * d[0]))
    })
}

/// What the camera shows while the sublevel's name is on screen, as (ahead, side) corners.
pub fn comedown_corners(aspect: f32) -> [(f32, f32); 4] {
    cutscene_corners(&COMEDOWN_CAMERA, aspect)
}

/// Which way the camera faces when you land.
///
/// Each floor starts both captains' cameras at `camera_angle` in `/user/Kando/aiConstants.txt`
/// (290°: behind the captain at 290°, looking toward 110°), and the captains face 0° (+Z), since
/// caves have no map rotation (`BaseGameSection::setupFloatMemory`). The cave then makes the active
/// captain the player; `CameraMgr::changePlayerMode` centers the camera behind him only when that
/// switches from Olimar, so with Louie it looks toward his cursor, straight ahead of him: 0°.
/// Pressing L does the same, but by then the captain faces the way the pod does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LandingCamera {
    /// After pressing L: the way the pod faces. What speedruns show.
    Pod,
    /// Olimar without L: toward 110°.
    Olimar,
    /// Louie without L: toward 0° (+Z).
    Louie,
}

/// A view drawn alone, turned as on screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Landing(LandingCamera),
    Comedown,
}

impl View {
    /// `pod` (or `landing`), `olimar`, `louie` or `comedown`.
    pub fn parse(s: &str) -> Result<Self, String> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "pod" | "landing" => View::Landing(LandingCamera::Pod),
            "olimar" => View::Landing(LandingCamera::Olimar),
            "louie" => View::Landing(LandingCamera::Louie),
            "comedown" => View::Comedown,
            _ => return Err(format!("unknown view {s:?}: pod, olimar, louie or comedown")),
        })
    }
}

impl LandingCamera {

    /// The direction the camera looks, in degrees, for a pod facing `pod`.
    pub fn heading(self, pod: f32) -> f32 {
        match self {
            LandingCamera::Pod => pod,
            LandingCamera::Olimar => 110.0,
            LandingCamera::Louie => 0.0,
        }
    }
}

/// The landing view's corners in game units, for a pod at (x, z) facing `pod` degrees. The
/// camera aims at the captain, who lands `look_ahead` units ahead of the pod (behind it).
pub fn landing_corners(x: f32, z: f32, pod: f32, camera: LandingCamera, aspect: f32) -> [(f32, f32); 4] {
    let (a, h) = (pod.to_radians(), camera.heading(pod).to_radians());
    let look = LANDING_CAMERA.look_ahead;
    // The point the trapezoid is measured from: the captain, moved back along the view.
    let (qx, qz) = (x + look * (a.sin() - h.sin()), z + look * (a.cos() - h.cos()));
    place_corners(qx, qz, camera.heading(pod), landing_shape(aspect).corners())
}

/// Corners given as (ahead, side) from a pod at (x, z) facing `degrees`, in game units.
fn place_corners(x: f32, z: f32, degrees: f32, corners: [(f32, f32); 4]) -> [(f32, f32); 4] {
    let a = degrees.to_radians();
    let f = (a.sin(), a.cos());
    let r = (-f.1, f.0);
    corners.map(|(ahead, side)| (x + f.0 * ahead + r.0 * side, z + f.1 * ahead + r.1 * side))
}

/// A view drawn alone: its corners on the floor (bottom-left, bottom-right, top-right, top-left
/// of the screen), turned so the screen's top edge is level.
#[derive(Clone, Copy)]
struct ScreenView {
    corners: [(f32, f32); 4],
    center: (f32, f32),
    /// Radians that turn the floor as the screen shows it.
    turn: f32,
    /// Where the turned view's top-left corner is, from `center`, and its size, in game units,
    /// with a margin.
    left: f32,
    top: f32,
    w: f32,
    h: f32,
}

impl ScreenView {
    const MARGIN: f32 = 40.0;

    fn new(corners: [(f32, f32); 4]) -> Self {
        let (tl, tr) = (corners[3], corners[2]);
        let turn = -(tr.1 - tl.1).atan2(tr.0 - tl.0);
        let center = (
            corners.iter().map(|c| c.0).sum::<f32>() / 4.0,
            corners.iter().map(|c| c.1).sum::<f32>() / 4.0,
        );
        let (sin, cos) = turn.sin_cos();
        let turned = corners.map(|(x, z)| {
            let (dx, dz) = (x - center.0, z - center.1);
            (dx * cos - dz * sin, dx * sin + dz * cos)
        });
        let min = |f: fn(&(f32, f32)) -> f32| turned.iter().map(f).fold(f32::INFINITY, f32::min);
        let max = |f: fn(&(f32, f32)) -> f32| turned.iter().map(f).fold(f32::NEG_INFINITY, f32::max);
        let (left, top) = (min(|c| c.0) - Self::MARGIN, min(|c| c.1) - Self::MARGIN);
        ScreenView {
            corners,
            center,
            turn,
            left,
            top,
            w: max(|c| c.0) + Self::MARGIN - left,
            h: max(|c| c.1) + Self::MARGIN - top,
        }
    }
}

// ---- Images and the font ----

/// Icons and radar images from the data folder, loaded once each.
pub struct Assets {
    data: PathBuf,
    font: fontdue::Font,
    images: Mutex<HashMap<String, Option<Arc<Pixmap>>>>,
    /// Folder listings, lowercase name -> file name, since file names' case varies.
    dirs: Mutex<HashMap<PathBuf, HashMap<String, String>>>,
}

// Teki whose icons aren't in the game's enemy icon set (as on the web page).
const SPECIAL_TEKI: &[(&str, &str)] = &[
    ("gashiba", "Gas_pipe_icon"),
    ("daiodogreen", "daiodogreen"),
    ("ooinu_s", "ooinu_s"),
    ("kareooinu_s", "kareooinu_s"),
    ("kareooinu_l", "kareooinu"),
    ("elechiba", "Electrical_wire_icon"),
    ("hiba", "Fire_geyser_icon"),
    ("bomb", "Bingo_Battle_Bomb_icon"),
    ("egg", "36px-Egg_icon"),
    ("wakame_s", "wakame_s"),
    ("chiyogami", "chiyogami"),
    ("rock", "Roulette_Wheel_boulder"),
    ("panhouse", "ooinu_s"),
];

impl Assets {
    /// Assets from a data folder holding `assets/<game>` and `resources`.
    pub fn new(data: &Path) -> Result<Assets, String> {
        let font_path = data.join("resources/BalooChettan2-ExtraBold.ttf");
        let bytes = std::fs::read(&font_path).map_err(|e| format!("{}: {e}", font_path.display()))?;
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?;
        Ok(Assets {
            data: data.to_path_buf(),
            font,
            images: Mutex::default(),
            dirs: Mutex::default(),
        })
    }

    /// The file `name` in `dir` (relative to the data folder), ignoring case.
    fn find(&self, dir: &str, name: &str) -> Option<PathBuf> {
        let dir = self.data.join(dir);
        let mut dirs = self.dirs.lock().unwrap();
        let listing = dirs.entry(dir.clone()).or_insert_with(|| {
            std::fs::read_dir(&dir)
                .map(|rd| {
                    rd.flatten()
                        .map(|e| {
                            let n = e.file_name().to_string_lossy().into_owned();
                            (n.to_ascii_lowercase(), n)
                        })
                        .collect()
                })
                .unwrap_or_default()
        });
        listing.get(&name.to_ascii_lowercase()).map(|n| dir.join(n))
    }

    /// The first of `candidates` (dir, file name) that exists, loaded and prepared by `prepare`.
    fn image(&self, key: &str, candidates: &[(String, String)], prepare: fn(&mut Pixmap)) -> Option<Arc<Pixmap>> {
        if let Some(img) = self.images.lock().unwrap().get(key) {
            return img.clone();
        }
        let img = candidates
            .iter()
            .find_map(|(dir, name)| self.find(dir, name))
            .and_then(|p| Pixmap::load_png(p).ok())
            .map(|mut p| {
                prepare(&mut p);
                Arc::new(p)
            });
        self.images.lock().unwrap().insert(key.to_string(), img.clone());
        img
    }

    pub fn unit(&self, name: &str, game: &str) -> Option<Arc<Pixmap>> {
        let n = name.to_ascii_lowercase();
        // The unit's folder name can differ in case from its name; find() handles both levels.
        let candidates: Vec<(String, String)> = [game, "pikmin2"]
            .iter()
            .filter_map(|g| {
                let folder = self.find(&format!("assets/{g}/mapunits"), &n)?;
                let rel = folder.strip_prefix(&self.data).ok()?.to_string_lossy().replace('\\', "/");
                Some((format!("{rel}/arc"), "texture.png".to_string()))
            })
            .collect();
        self.image(&format!("unit/{game}/{n}"), &candidates, prepare_unit)
    }

    pub fn teki(&self, name: &str, game: &str) -> Option<Arc<Pixmap>> {
        let mut n = name.to_ascii_lowercase();
        if n == "fkabuto" {
            n = "kabuto".into();
        }
        if n == "fminihoudai" {
            n = "minihoudai".into();
        }
        if let Some((_, special)) = SPECIAL_TEKI.iter().find(|(k, _)| *k == n) {
            return self.special(special);
        }
        let file = format!("{n}.png");
        let candidates = [
            (format!("assets/{game}/teki"), file.clone()),
            ("assets/pikmin2/teki".to_string(), file),
        ];
        self.image(&format!("teki/{game}/{n}"), &candidates, |_| {})
    }

    pub fn treasure(&self, name: &str, game: &str) -> Option<Arc<Pixmap>> {
        let n = name.to_ascii_lowercase();
        let file = format!("{n}.png");
        let candidates = [
            (format!("assets/{game}/treasures"), file.clone()),
            ("assets/pikmin2/treasures".to_string(), file),
        ];
        self.image(&format!("treasure/{game}/{n}"), &candidates, |_| {})
    }

    /// CaveGen's "pretty" picture of a vanilla map unit, or a floor background.
    pub fn pretty(&self, name: &str) -> Option<Arc<Pixmap>> {
        let key = format!("pretty/{}", name.to_ascii_lowercase());
        if let Some(img) = self.images.lock().unwrap().get(&key) {
            return img.clone();
        }
        let img = self
            .find("resources/pretty", &format!("{name}.webp"))
            .and_then(|p| load_webp(&p))
            .map(Arc::new);
        self.images.lock().unwrap().insert(key, img.clone());
        img
    }

    pub fn special(&self, name: &str) -> Option<Arc<Pixmap>> {
        let candidates = [("resources/enemytex_special".to_string(), format!("{name}.png"))];
        self.image(&format!("special/{}", name.to_ascii_lowercase()), &candidates, |_| {})
    }

    /// A special icon recolored to one color, keeping its shape (Caveripper's Colorize).
    fn special_tinted(&self, name: &str, color: Color) -> Option<Arc<Pixmap>> {
        let c = color.to_color_u8();
        let key = format!("tinted/{}/{:?}", name.to_ascii_lowercase(), (c.red(), c.green(), c.blue()));
        if let Some(img) = self.images.lock().unwrap().get(&key) {
            return img.clone();
        }
        let img = self.special(name).map(|base| {
            let mut p = (*base).clone();
            for px in p.pixels_mut() {
                let a = px.alpha();
                *px = tiny_skia::ColorU8::from_rgba(c.red(), c.green(), c.blue(), a).premultiply();
            }
            Arc::new(p)
        });
        self.images.lock().unwrap().insert(key, img.clone());
        img
    }
}

/// Decodes a WebP picture into a premultiplied pixmap.
fn load_webp(path: &Path) -> Option<Pixmap> {
    let img = image::open(path).ok()?.to_rgba8();
    let mut p = Pixmap::new(img.width(), img.height())?;
    for (px, c) in p.pixels_mut().iter_mut().zip(img.pixels()) {
        *px = tiny_skia::ColorU8::from_rgba(c[0], c[1], c[2], c[3]).premultiply();
    }
    Some(p)
}

/// Radar images: opaque ones (CaveGen's, used for the romhacks) get their shading as alpha like
/// Caveripper's, then everything is brightened 1.6× as on the web page.
fn prepare_unit(p: &mut Pixmap) {
    let opaque = p.pixels().iter().all(|c| c.alpha() == 255);
    for px in p.pixels_mut() {
        let c = px.demultiply();
        let a = if opaque { c.red() } else { c.alpha() };
        let b = |v: u8| (v as f32 * 1.6).min(255.0) as u8;
        *px = tiny_skia::ColorU8::from_rgba(b(c.red()), b(c.green()), b(c.blue()), a).premultiply();
    }
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let v = |i: usize, n: usize| u8::from_str_radix(&s[i..i + n], 16).unwrap();
    if s.len() == 3 {
        Color::from_rgba8(v(0, 1) * 17, v(1, 1) * 17, v(2, 1) * 17, 255)
    } else {
        Color::from_rgba8(v(0, 2), v(2, 2), v(4, 2), 255)
    }
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba8(r, g, b, (a * 255.0).round() as u8)
}

// Caveripper's spawn group colors (groups 0-9), for its spawn point markers.
const SP_COLORS: [&str; 10] = [
    "#fa57cf", "#c90234", "#e67300", "", "", "#858585", "#3b945a", "#e63256", "#59068a", "#2dada7",
];

// CaveGen's colors for spawn point groups 0-9, and treasure gauge rings.
const SPAWN_COLORS: [&str; 10] = [
    "#ff82dc", "#dc0000", "#dc7800", "#000", "#008c3c", "#505050", "#8ce164", "#009696", "#a000a0", "#0000ff",
];
const GAUGE_TICKS: [f32; 10] = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
// The treasure gauge's bands, as on the web page (`GAUGE_SHADES` in web/app.js): the needle sits at
// 0.5 + 9.5 (1 - d/900)^2 tenths, d being the flat distance minus the treasure's radius.
const GAUGE_SHADES: [(u8, u8, u8, f32); 10] = [
    (255, 240, 150, 0.16),
    (255, 228, 110, 0.22),
    (255, 214, 70, 0.28),
    (255, 170, 60, 0.30),
    (252, 152, 45, 0.35),
    (248, 134, 35, 0.40),
    (242, 116, 30, 0.45),
    (236, 98, 25, 0.50),
    (220, 40, 40, 0.55),
    (190, 15, 40, 0.62),
];

/// CaveGen's QuickGlance disc color for an object, at alpha 100/255, or None.
fn quick_glance_color(o: &Object, colossal: bool) -> Option<Color> {
    let a = 100.0 / 255.0;
    let name = o.name().to_ascii_lowercase();
    let c = |r, g, b| Some(rgba(r, g, b, a));
    match o.kind.as_str() {
        "ship" => c(255, 30, 30),
        "hole" | "geyser" => c(0, 140, 60),
        "onion" => match o.color.unwrap_or(0) {
            0 => c(0, 0, 255),
            1 => c(255, 0, 0),
            2 => c(255, 255, 0),
            3 => c(160, 0, 255),
            _ => c(60, 60, 60),
        },
        // Colossal Caverns marks only its buried treasures, in green.
        "treasure" if colossal => {
            if o.buried {
                c(60, 255, 120)
            } else {
                None
            }
        }
        "treasure" => c(220, 120, 0),
        "teki" => {
            if (o.carrying.is_some() && (name != "blackman" || colossal)) || name == "bigtreasure" {
                c(220, 120, 0)
            } else if name == "minihoudai" || (colossal && name == "kumachappy") {
                c(255, 130, 220)
            } else if colossal && name == "leafchappy" {
                c(60, 255, 120)
            } else if name == "blackpom" {
                c(160, 0, 255)
            } else if name == "whitepom" {
                c(60, 60, 60)
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---- Drawing ----

struct Canvas<'a> {
    pixmap: Pixmap,
    /// Map pixels (game units / UNIT * cell) to image pixels.
    base: Transform,
    mask: Option<Mask>,
    /// Degrees that turn icons and numbers upright again in the landing view.
    upright: f32,
    /// Opacity for icons (empty spawn points are faded).
    alpha: f32,
    assets: &'a Assets,
}

fn paint(color: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(color);
    p.anti_alias = true;
    p
}

fn stroke(width: f32) -> Stroke {
    Stroke {
        width,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        ..Stroke::default()
    }
}

impl Canvas<'_> {
    fn fill_circle(&mut self, x: f32, z: f32, r: f32, color: Color) {
        if let Some(path) = PathBuilder::from_circle(x, z, r) {
            self.pixmap
                .fill_path(&path, &paint(color), FillRule::Winding, self.base, self.mask.as_ref());
        }
    }

    fn stroke_circle(&mut self, x: f32, z: f32, r: f32, color: Color, width: f32) {
        if let Some(path) = PathBuilder::from_circle(x, z, r) {
            self.pixmap
                .stroke_path(&path, &paint(color), &stroke(width), self.base, self.mask.as_ref());
        }
    }

    fn fill_rect(&mut self, x: f32, z: f32, w: f32, h: f32, color: Color) {
        if let Some(rect) = Rect::from_xywh(x, z, w, h) {
            self.pixmap.fill_rect(rect, &paint(color), self.base, self.mask.as_ref());
        }
    }

    fn stroke_path(&mut self, pb: PathBuilder, color: Color, s: &Stroke) {
        if let Some(path) = pb.finish() {
            self.pixmap.stroke_path(&path, &paint(color), s, self.base, self.mask.as_ref());
        }
    }

    /// Draws `img` centered at (x, z) in map pixels, fitted into a `size` square and turned
    /// `rot` degrees (upright when None). Returns whether there was an image.
    fn icon(&mut self, img: Option<Arc<Pixmap>>, x: f32, z: f32, size: f32, rot: Option<f32>) -> bool {
        let Some(img) = img else { return false };
        let r = (size / img.width() as f32).min(size / img.height() as f32);
        let t = self
            .base
            .pre_translate(x, z)
            .pre_concat(Transform::from_rotate(rot.unwrap_or(self.upright)))
            .pre_scale(r, r)
            .pre_translate(-(img.width() as f32) / 2.0, -(img.height() as f32) / 2.0);
        let pp = PixmapPaint {
            quality: FilterQuality::Bicubic,
            opacity: self.alpha,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(0, 0, img.as_ref().as_ref(), &pp, t, self.mask.as_ref());
        true
    }

    /// Text with a black outline at (x, z), centered vertically and either centered or (with
    /// `left`) starting there, like the web page's labels.
    fn label(&mut self, text: &str, x: f32, z: f32, color: Color, size: f32, left: bool) {
        let outline = (size / 4.0).max(2.0);
        if let Some((img, ax, ay)) = text_image(&self.assets.font, text, size, color, Some(outline)) {
            self.draw_text(&img, x, z, ax, ay, left);
        }
    }

    fn draw_text(&mut self, img: &Pixmap, x: f32, z: f32, (ax_left, ax_center): (f32, f32), ay: f32, left: bool) {
        let ax = if left { ax_left } else { ax_center };
        let t = self
            .base
            .pre_translate(x, z)
            .pre_concat(Transform::from_rotate(self.upright))
            .pre_translate(-ax, -ay);
        let quality = if self.upright == 0.0 {
            FilterQuality::Nearest
        } else {
            FilterQuality::Bilinear
        };
        // Whole pixels when upright, so the text stays sharp.
        let t = if self.upright == 0.0 {
            Transform::from_translate(t.tx.round(), t.ty.round())
        } else {
            t
        };
        let pp = PixmapPaint {
            quality,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(0, 0, img.as_ref(), &pp, t, self.mask.as_ref());
    }

    /// A colored disc with the start of `text` on it, for objects with no icon.
    fn fallback(&mut self, text: &str, x: f32, z: f32, size: f32, color: Color) {
        self.fill_circle(x, z, size / 2.5, color);
        let short: String = text.chars().take(3).collect();
        if let Some((img, ax, ay)) = text_image(&self.assets.font, &short, (size / 3.2).max(8.0), Color::WHITE, None) {
            self.draw_text(&img, x, z, ax, ay, false);
        }
    }

    fn arrow(&mut self, x1: f32, z1: f32, x2: f32, z2: f32, head: f32, color: Color, width: f32) {
        let a = (z2 - z1).atan2(x2 - x1);
        let mut pb = PathBuilder::new();
        pb.move_to(x1, z1);
        pb.line_to(x2, z2);
        pb.move_to(x2, z2);
        pb.line_to(x2 - head * (a - 0.4).cos(), z2 - head * (a - 0.4).sin());
        pb.move_to(x2, z2);
        pb.line_to(x2 - head * (a + 0.4).cos(), z2 - head * (a + 0.4).sin());
        self.stroke_path(pb, color, &stroke(width));
    }

    /// An arrow for a facing angle in degrees (0 = +z, like the game).
    fn facing(&mut self, x: f32, z: f32, degrees: f32, len: f32, color: Color) {
        let a = degrees.to_radians();
        let (x2, z2) = (x + len * a.sin(), z + len * a.cos());
        self.arrow(x, z, x2, z2, len * 0.35, Color::BLACK, 3.0);
        self.arrow(x, z, x2, z2, len * 0.35, color, 1.5);
    }
}

/// Renders `text` with an optional black outline of total width `outline`. Returns the image
/// and the anchor points: x for left-aligned and centered text, and y for the vertical middle.
fn text_image(font: &fontdue::Font, text: &str, size: f32, color: Color, outline: Option<f32>) -> Option<(Pixmap, (f32, f32), f32)> {
    let lm = font.horizontal_line_metrics(size)?;
    let pad = outline.map_or(1.0, |o| (o / 2.0).ceil() + 1.0);
    let glyphs: Vec<(fontdue::Metrics, Vec<u8>)> = text.chars().map(|c| font.rasterize(c, size)).collect();
    let advance: f32 = glyphs.iter().map(|(m, _)| m.advance_width).sum();
    let w = (advance + 2.0 * pad).ceil() as usize + 1;
    let h = (lm.ascent - lm.descent + 2.0 * pad).ceil() as usize + 1;
    let baseline = pad + lm.ascent;
    let mut cover = vec![0f32; w * h];
    let mut pen = pad;
    for (m, bitmap) in &glyphs {
        let gx = (pen + m.xmin as f32).round() as i64;
        let gy = (baseline - m.height as f32 - m.ymin as f32).round() as i64;
        for row in 0..m.height {
            for col in 0..m.width {
                let (px, py) = (gx + col as i64, gy + row as i64);
                if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
                    let i = py as usize * w + px as usize;
                    cover[i] = cover[i].max(bitmap[row * m.width + col] as f32 / 255.0);
                }
            }
        }
        pen += m.advance_width;
    }
    // The outline: the text's coverage spread by half the outline's width.
    let halo = match outline {
        Some(o) => {
            let r = o / 2.0;
            let ri = r.ceil() as i64;
            let mut out = vec![0f32; w * h];
            for y in 0..h as i64 {
                for x in 0..w as i64 {
                    let mut best = 0f32;
                    for dy in -ri..=ri {
                        for dx in -ri..=ri {
                            let d = ((dx * dx + dy * dy) as f32).sqrt();
                            if d > r + 0.5 {
                                continue;
                            }
                            let (sx, sy) = (x + dx, y + dy);
                            if sx < 0 || sy < 0 || sx >= w as i64 || sy >= h as i64 {
                                continue;
                            }
                            let edge = (r + 0.5 - d).min(1.0);
                            best = best.max(cover[sy as usize * w + sx as usize] * edge);
                        }
                    }
                    out[y as usize * w + x as usize] = best;
                }
            }
            out
        }
        None => vec![0f32; w * h],
    };
    let mut img = Pixmap::new(w as u32, h as u32)?;
    let c = color.to_color_u8();
    for (i, px) in img.pixels_mut().iter_mut().enumerate() {
        // Text over its outline, both "source over".
        let ta = cover[i];
        let ha = halo[i] * (1.0 - ta);
        let a = ta + ha;
        if a <= 0.0 {
            continue;
        }
        let ch = |v: u8| (v as f32 * ta / a).round() as u8;
        let unpremul = tiny_skia::ColorU8::from_rgba(ch(c.red()), ch(c.green()), ch(c.blue()), (a * 255.0).round() as u8);
        *px = unpremul.premultiply();
    }
    Some((img, (pad, pad + advance / 2.0), baseline - (lm.ascent + lm.descent) / 2.0))
}

/// Draws a layout. The image is the whole cave with a half-cell margin, or just the landing view.
pub fn render(layout: &Layout, opts: &Options, assets: &Assets) -> Result<Pixmap, String> {
    let cell = CELL * opts.scale;
    let to_px = |v: f32| v / UNIT * cell;
    let ship = layout.objects.iter().find(|o| o.kind == "ship");
    // The landing or comedown view alone: the floor in view, turned as it is on screen.
    let single = match opts.view {
        Some(view) => {
            let ship = ship.ok_or("this sublevel has no pod, so it has no landing or comedown view")?;
            let angle = ship.angle.unwrap_or(0.0);
            Some(match view {
                View::Landing(camera) => landing_corners(ship.x, ship.z, angle, camera, opts.aspect),
                View::Comedown => place_corners(ship.x, ship.z, angle, comedown_corners(opts.aspect)),
            })
        }
        None => None,
    };
    // The landing and comedown outlines that are on, in game units.
    let outlines: Vec<([(f32, f32); 4], &str)> = match ship.filter(|s| s.angle.is_some() && single.is_none()) {
        Some(ship) => {
            let angle = ship.angle.unwrap();
            let landing = |camera| landing_corners(ship.x, ship.z, angle, camera, opts.aspect);
            let mut out = vec![];
            if opts.comedown_box {
                out.push((place_corners(ship.x, ship.z, angle, comedown_corners(opts.aspect)), COMEDOWN_COLOR));
            }
            for (on, camera, color) in [
                (opts.landing_box_olimar, LandingCamera::Olimar, OLIMAR_COLOR),
                (opts.landing_box_louie, LandingCamera::Louie, LOUIE_COLOR),
                (opts.landing_box, LandingCamera::Pod, LANDING_COLOR),
            ] {
                if on {
                    out.push((landing(camera), color));
                }
            }
            out
        }
        None => vec![],
    };
    // The cave with a half-cell margin, grown to fit the outlines (in grid cells).
    let (mut x0, mut z0, mut x1, mut z1) = (-0.5f32, -0.5f32, layout.width + 0.5, layout.height + 0.5);
    for (x, z) in outlines.iter().flat_map(|o| o.0) {
        x0 = x0.min(x / UNIT - 0.25);
        z0 = z0.min(z / UNIT - 0.25);
        x1 = x1.max(x / UNIT + 0.25);
        z1 = z1.max(z / UNIT + 0.25);
    }
    let view = single.map(ScreenView::new);
    let (w, h) = match view {
        Some(v) => (to_px(v.w), to_px(v.h)),
        None => ((x1 - x0) * cell, (z1 - z0) * cell),
    };
    let mut pixmap = Pixmap::new(w.round().max(1.0) as u32, h.round().max(1.0) as u32).ok_or("the image would be too large")?;
    pixmap.fill(hex("#0f0f0f"));

    let (base, mask, upright) = match view {
        Some(v) => {
            // Turn as the screen is, and show only what's in view.
            let base = Transform::from_translate(-to_px(v.left), -to_px(v.top))
                .pre_concat(Transform::from_rotate(v.turn.to_degrees()))
                .pre_translate(-to_px(v.center.0), -to_px(v.center.1));
            let mut pb = PathBuilder::new();
            for (i, (x, z)) in v.corners.into_iter().enumerate() {
                if i == 0 {
                    pb.move_to(to_px(x), to_px(z));
                } else {
                    pb.line_to(to_px(x), to_px(z));
                }
            }
            pb.close();
            let mut mask = Mask::new(pixmap.width(), pixmap.height()).ok_or("bad mask")?;
            mask.fill_path(&pb.finish().ok_or("bad view shape")?, FillRule::Winding, true, base);
            (base, Some(mask), -v.turn.to_degrees())
        }
        None => (Transform::from_translate(-x0 * cell, -z0 * cell), None, 0.0),
    };
    let mut cv = Canvas {
        pixmap,
        base,
        mask,
        upright,
        alpha: 1.0,
        assets,
    };

    // Pretty: CaveGen's floor backdrop for toy and grassy caves, under the pictures.
    if opts.pretty {
        let first = layout.units.first().map(|u| u.name.to_ascii_lowercase()).unwrap_or_default();
        let bg = if first.contains("toy") {
            assets.pretty("toy_bg")
        } else if first.contains("kusachi") {
            assets.pretty("kusachi_bg")
        } else {
            None
        };
        if let Some(bg) = bg {
            let pattern = tiny_skia::Pattern::new(
                bg.as_ref().as_ref(),
                tiny_skia::SpreadMode::Repeat,
                FilterQuality::Bilinear,
                1.0,
                // Fixed to the floor: 40 of the backdrop's pixels per map cell, as in CaveGen.
                Transform::from_scale(cell / 40.0, cell / 40.0),
            );
            let paint = Paint {
                shader: pattern,
                ..Paint::default()
            };
            if let Some(rect) = Rect::from_xywh(0.0, 0.0, layout.width * cell, layout.height * cell) {
                cv.pixmap.fill_rect(rect, &paint, cv.base, cv.mask.as_ref());
            }
        }
    }

    // Map units, turned clockwise in quarter turns like the game's radar images.
    for u in &layout.units {
        let (cx, cz) = ((u.x + u.w / 2.0) * cell, (u.z + u.h / 2.0) * cell);
        let pretty = if opts.pretty { assets.pretty(&u.name) } else { None };
        let is_pretty = pretty.is_some();
        match pretty.or_else(|| assets.unit(&u.name, &u.game)) {
            Some(img) => {
                let (w0, h0) = if u.rot % 2 == 1 { (u.h, u.w) } else { (u.w, u.h) };
                // A picture is drawn a pixel larger on every side, so neighbors overlap rather than
                // leave a hairline where the smoothed edges fade.
                let extra = if is_pretty { 1.0 } else { 0.0 };
                let t = cv
                    .base
                    .pre_translate(cx, cz)
                    .pre_concat(Transform::from_rotate(u.rot as f32 * 90.0))
                    .pre_translate(-w0 * cell / 2.0 - extra, -h0 * cell / 2.0 - extra)
                    .pre_scale((w0 * cell + 2.0 * extra) / img.width() as f32, (h0 * cell + 2.0 * extra) / img.height() as f32);
                let pp = PixmapPaint {
                    quality: if is_pretty {
                        FilterQuality::Bilinear
                    } else {
                        FilterQuality::Nearest
                    },
                    ..PixmapPaint::default()
                };
                cv.pixmap.draw_pixmap(0, 0, img.as_ref().as_ref(), &pp, t, cv.mask.as_ref());
            }
            None => cv.fill_rect(cx - u.w * cell / 2.0, cz - u.h * cell / 2.0, u.w * cell, u.h * cell, hex("#333")),
        }
    }

    if opts.water {
        for u in &layout.units {
            let (cx, cz) = ((u.x + u.w / 2.0) * cell, (u.z + u.h / 2.0) * cell);
            for [x1, z1, x2, z2] in &u.water {
                cv.fill_rect(
                    cx + to_px(*x1),
                    cz + to_px(*z1),
                    to_px(x2 - x1),
                    to_px(z2 - z1),
                    rgba(0, 100, 230, 0.25),
                );
            }
        }
    }

    if opts.grid {
        let mut pb = PathBuilder::new();
        for x in 0..=layout.width as i32 {
            pb.move_to(x as f32 * cell, 0.0);
            pb.line_to(x as f32 * cell, layout.height * cell);
        }
        for z in 0..=layout.height as i32 {
            pb.move_to(0.0, z as f32 * cell);
            pb.line_to(layout.width * cell, z as f32 * cell);
        }
        cv.stroke_path(pb, rgba(255, 255, 255, 0.12), &stroke(1.0));
    }

    let thin = (cell / 30.0).max(1.0);
    let label_size = (cell * 0.28).max(9.0);
    let carried: Vec<&Object> = layout
        .objects
        .iter()
        .filter(|o| o.kind == "treasure" || (o.kind == "teki" && o.carrying.is_some()))
        .collect();

    if opts.gauge && !carried.is_empty() {
        // On its own layer, from the outermost band in: each band clears what's inside it and
        // paints its own shade; then the layer goes on the map.
        if let Some(mut layer) = Pixmap::new(cv.pixmap.width(), cv.pixmap.height()) {
            for (i, tick) in GAUGE_TICKS.iter().enumerate() {
                let reach = 900.0 * (1.0 - ((tick - 0.5) / 9.5).sqrt());
                let mut pb = PathBuilder::new();
                for o in &carried {
                    // A teki's own size isn't known, so treasures it carries use no extra radius.
                    let extra = if o.kind == "treasure" { o.radius.unwrap_or(0.0) } else { 0.0 };
                    pb.push_circle(to_px(o.x), to_px(o.z), to_px(reach + extra));
                }
                let Some(path) = pb.finish() else { continue };
                let mut clear = paint(Color::BLACK);
                clear.blend_mode = tiny_skia::BlendMode::Clear;
                layer.fill_path(&path, &clear, FillRule::Winding, cv.base, None);
                let (r, g, b, a) = GAUGE_SHADES[i];
                layer.fill_path(&path, &paint(rgba(r, g, b, a)), FillRule::Winding, cv.base, None);
            }
            cv.pixmap.draw_pixmap(
                0,
                0,
                layer.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                cv.mask.as_ref(),
            );
        }
    }

    if opts.quick_glance {
        let colossal = layout.sublevel.to_ascii_lowercase().starts_with("colossal");
        for o in &layout.objects {
            if let Some(c) = quick_glance_color(o, colossal) {
                cv.fill_circle(to_px(o.x), to_px(o.z), cell * 0.625, c);
            }
        }
    }

    if opts.door_links {
        let mut pb = PathBuilder::new();
        for d in &layout.doors {
            for &j in &d.links {
                if let Some(other) = layout.doors.get(j) {
                    pb.move_to(to_px(d.x), to_px(d.z));
                    pb.line_to(to_px(other.x), to_px(other.z));
                }
            }
        }
        cv.stroke_path(pb, rgba(255, 0, 255, 0.45), &stroke(thin));
    }

    if opts.waypoints {
        for wp in &layout.waypoints {
            let r = to_px(wp.r).max(2.0);
            cv.fill_circle(to_px(wp.x), to_px(wp.z), r, rgba(90, 170, 255, 0.15));
            cv.stroke_circle(to_px(wp.x), to_px(wp.z), r, rgba(90, 170, 255, 0.8), thin);
        }
        for wp in &layout.waypoints {
            if let Some(n) = wp.next.and_then(|n| layout.waypoints.get(n)) {
                cv.arrow(
                    to_px(wp.x),
                    to_px(wp.z),
                    to_px(n.x),
                    to_px(n.z),
                    (cell * 0.12).max(4.0),
                    rgba(120, 200, 255, 0.95),
                    thin,
                );
            }
        }
    }

    if opts.paths {
        let s = Stroke {
            line_join: LineJoin::Round,
            ..stroke((cell / 18.0).max(1.5))
        };
        for o in &carried {
            let Some(path) = o.path.as_ref().filter(|p| p.len() >= 4) else {
                continue;
            };
            let mut pb = PathBuilder::new();
            pb.move_to(to_px(path[0]), to_px(path[1]));
            for p in path[2..].as_chunks::<2>().0 {
                pb.line_to(to_px(p[0]), to_px(p[1]));
            }
            cv.stroke_path(pb, rgba(255, 200, 40, 0.9), &s);
        }
    }

    if opts.spawnpoints {
        // Caveripper's markers: rings for easy teki, a dot for hard teki, a duck for treasures, the
        // exit icon for the hole, a leaf for plants, the ship for the pod and a star for special teki.
        let mut points: Vec<&Spawnpoint> = layout.spawnpoints.iter().collect();
        points.sort_by_key(|sp| sp.group);
        for sp in points {
            let (x, z) = (to_px(sp.x), to_px(sp.z));
            let alpha = if sp.used { 1.0 } else { 0.55 };
            cv.alpha = alpha;
            let faded = |c: &str| {
                let mut c = hex(c);
                c.apply_opacity(alpha);
                c
            };
            let drawn = match sp.group {
                0 => {
                    let mut r = to_px(sp.r).max(3.0);
                    for _ in 0..3 {
                        cv.stroke_circle(x, z, r, faded(SP_COLORS[0]), (cell / 40.0).max(1.0));
                        r *= 0.85;
                    }
                    true
                }
                1 => {
                    cv.fill_circle(x, z, (cell * 0.11).max(2.5), faded(SP_COLORS[1]));
                    true
                }
                2 => cv.icon(assets.special_tinted("duck", hex(SP_COLORS[2])), x, z, cell * 0.42, None),
                4 => cv.icon(assets.special("cave_white"), x, z, cell * 0.42, None),
                6 => cv.icon(assets.special_tinted("leaf_icon", hex(SP_COLORS[6])), x, z, cell * 0.4, None),
                7 => cv.icon(assets.special("ship"), x, z, cell * 0.42, None),
                8 => cv.icon(assets.special_tinted("star", hex(SP_COLORS[8])), x, z, cell * 0.5, None),
                _ => false,
            };
            if !drawn {
                let c = SP_COLORS
                    .get(sp.group)
                    .copied()
                    .filter(|c| !c.is_empty())
                    .unwrap_or(SPAWN_COLORS.get(sp.group).copied().unwrap_or("#fff"));
                cv.fill_circle(x, z, (cell * 0.09).max(2.5), faded(c));
            }
            cv.alpha = 1.0;
            if opts.angles {
                cv.facing(x, z, sp.angle, cell * 0.35, Color::WHITE);
            }
        }
    }

    // Exits and the ship first, then gates, treasures and teki on top.
    let order = |k: &str| match k {
        "hole" | "geyser" => 0,
        "ship" | "onion" => 1,
        "gate" => 2,
        "treasure" => 3,
        _ => 4,
    };
    let mut objects: Vec<&Object> = layout.objects.iter().filter(|o| opts.shows(o)).collect();
    objects.sort_by_key(|o| order(&o.kind));
    for o in objects {
        let (x, z) = (to_px(o.x), to_px(o.z));
        match o.kind.as_str() {
            "ship" => {
                let size = cell * 0.6;
                if !cv.icon(assets.special("pod_icon"), x, z, size, None) {
                    cv.fallback("Pod", x, z, size, hex("#c00"));
                }
            }
            "hole" => {
                let size = cell * 0.6;
                cv.icon(assets.special("Cave_icon"), x, z, size, None);
                if o.plugged {
                    cv.icon(assets.special("36px-Clog_icon"), x, z, size, None);
                }
            }
            "geyser" => {
                cv.icon(assets.special("Geyser_icon"), x, z, cell * 0.7, None);
            }
            "onion" => {
                cv.icon(assets.special(&format!("onion{}", o.color.unwrap_or(0))), x, z, cell * 0.45, None);
            }
            "gate" => {
                let rot = if o.dir.unwrap_or(0) % 2 == 1 { 90.0 } else { 0.0 };
                cv.icon(assets.special("Gray_bramble_gate_icon"), x, z, cell * 0.85, Some(rot));
                if opts.gate_health {
                    let hp = o.health.unwrap_or(0.0).round().to_string();
                    cv.label(&hp, x, z, Color::WHITE, (cell * 0.26).max(8.0), false);
                }
            }
            "treasure" => {
                let size = cell * 0.55;
                if !cv.icon(assets.treasure(o.name(), o.game()), x, z, size, None) {
                    cv.fallback(o.name(), x, z, size, hex("#c70"));
                }
            }
            "teki" => {
                let size = if o.group() == Some(0) { cell * 0.45 } else { cell * 0.6 };
                if !cv.icon(assets.teki(o.name(), o.game()), x, z, size, None) {
                    cv.fallback(o.name(), x, z, size, hex("#7a2a8a"));
                }
                if let Some(c) = &o.carrying {
                    cv.icon(assets.treasure(c, o.game()), x + size * 0.35, z + size * 0.35, size * 0.6, None);
                }
                if o.falling {
                    cv.icon(assets.special("falling_icon"), x - size * 0.4, z - size * 0.4, size * 0.4, None);
                }
                if let Some(n) = o.count.filter(|n| *n > 1) {
                    cv.label(
                        &format!("×{n}"),
                        x + size * 0.3,
                        z - size * 0.35,
                        Color::WHITE,
                        (size * 0.38).max(9.0),
                        true,
                    );
                }
            }
            _ => {}
        }
        if opts.angles
            && o.kind != "gate"
            && let Some(a) = o.angle
        {
            cv.facing(x, z, a, cell * 0.4, hex("#ffe14d"));
        }
    }

    // The pod's facing alone (CaveGen's -drawPodAngle); "angles" already draws it.
    if opts.pod_angle && !opts.angles {
        if let Some(ship) = layout.objects.iter().find(|o| o.kind == "ship") {
            if let Some(a) = ship.angle {
                cv.facing(to_px(ship.x), to_px(ship.z), a, cell * 0.6, hex("#ffe14d"));
            }
        }
    }

    // Numbers on top of everything.
    if opts.spawn_order {
        let mut pb = PathBuilder::new();
        for u in &layout.units {
            if let Some(r) = Rect::from_xywh(u.x * cell, u.z * cell, u.w * cell, u.h * cell) {
                pb.push_rect(r);
            }
        }
        cv.stroke_path(pb, rgba(0, 225, 225, 0.6), &stroke(thin));
        for (i, u) in layout.units.iter().enumerate() {
            cv.label(
                &i.to_string(),
                u.x * cell + cell * 0.08,
                (u.z + u.h) * cell - cell * 0.2,
                hex("#00ffff"),
                label_size,
                true,
            );
        }
        for o in layout.objects.iter().filter(|o| opts.shows(o)) {
            if let Some(n) = o.order {
                cv.label(
                    &n.to_string(),
                    to_px(o.x) - cell * 0.38,
                    to_px(o.z) + cell * 0.38,
                    hex("#00ffff"),
                    label_size,
                    false,
                );
            }
        }
    }
    if opts.path_dists {
        for o in carried.iter().filter(|o| opts.shows(o)) {
            if let Some(d) = o.path_dist {
                cv.label(
                    &d.to_string(),
                    to_px(o.x) + cell * 0.1,
                    to_px(o.z) - cell * 0.12,
                    hex("#c79bff"),
                    label_size,
                    true,
                );
            }
        }
    }
    if opts.unit_scores || opts.teki_scores {
        for u in &layout.units {
            let (cx, cz) = ((u.x + u.w / 2.0) * cell, (u.z + u.h / 2.0) * cell);
            if let Some(s) = u.score.filter(|s| opts.unit_scores && *s >= 0) {
                cv.label(&s.to_string(), cx, cz, Color::WHITE, label_size, false);
            }
            if let Some(s) = u.teki_score.filter(|s| opts.teki_scores && *s > 0) {
                cv.label(&s.to_string(), cx, cz + cell * 0.3, hex("#ff5a5a"), label_size, false);
            }
        }
    }
    if opts.spawn_scores {
        for sp in &layout.spawnpoints {
            let (x, z) = (to_px(sp.x) + cell * 0.15, to_px(sp.z));
            if (sp.group == 2 || sp.group == 9) && sp.treasure_score > -1 {
                cv.label(&sp.treasure_score.to_string(), x, z + cell * 0.12, hex("#ffaa00"), label_size, true);
            }
            if (sp.group == 4 || sp.group == 9) && sp.hole_score > -1 {
                cv.label(&sp.hole_score.to_string(), x, z - cell * 0.12, hex("#3cff3c"), label_size, true);
            }
        }
    }
    // The landing and comedown outlines.
    for (corners, color) in &outlines {
        let mut pb = PathBuilder::new();
        for (i, (x, z)) in corners.iter().enumerate() {
            if i == 0 {
                pb.move_to(to_px(*x), to_px(*z));
            } else {
                pb.line_to(to_px(*x), to_px(*z));
            }
        }
        pb.close();
        let mut s = stroke((cell / 16.0).max(1.5));
        s.dash = StrokeDash::new(vec![cell / 4.0, cell / 6.0], 0.0);
        cv.stroke_path(pb, hex(color), &s);
    }
    if opts.door_scores || opts.door_ids {
        let (mut unit, mut first) = (-1, 0);
        for (i, d) in layout.doors.iter().enumerate() {
            if d.unit != unit {
                unit = d.unit;
                first = i;
            }
            // Nudge labels into the unit, away from the seam.
            let (dx, dz) = [(0.0, 1.0), (-1.0, 0.0), (0.0, -1.0), (1.0, 0.0)][d.dir % 4];
            let (x, z) = (to_px(d.x) + dx * cell * 0.22, to_px(d.z) + dz * cell * 0.22);
            if opts.door_ids {
                cv.label(&(i - first).to_string(), x, z, hex("#b48cff"), label_size, false);
            }
            if opts.door_scores && d.score >= 0 {
                let dz2 = if opts.door_ids { cell * 0.25 } else { 0.0 };
                cv.label(&d.score.to_string(), x, z + dz2, hex("#5ad2ff"), label_size, false);
            }
        }
    }
    Ok(cv.pixmap)
}

// ---- Aggregator groups ----

/// The aggregator's result, as the web crate's `agg_result` describes it.
#[derive(Deserialize)]
pub struct AggResult {
    pub total: u64,
    pub groups_total: u64,
    pub dropped: u64,
    /// The landing view: near, far, near half-width, far half-width.
    pub view: [f32; 4],
    pub groups: Vec<AggGroup>,
}

#[derive(Deserialize)]
pub struct AggGroup {
    pub count: u64,
    pub seeds: Vec<u32>,
    /// The pod's position and facing in the group's frame, for landing groupings.
    pub pod: Option<[f32; 4]>,
    pub units: Vec<AggUnit>,
    /// Spots: position and (object id, how many seeds had it there).
    pub spots: Vec<(f32, f32, Vec<(String, u64)>)>,
    /// Half-cell grid positions other units covered, and how often.
    pub cells: Vec<(f32, f32, u64)>,
}

#[derive(Deserialize)]
pub struct AggUnit {
    pub name: String,
    pub game: String,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub h: f32,
    pub rot: u32,
}

/// What an aggregator group map shows.
pub struct AggOptions {
    pub treasures: bool,
    pub teki: bool,
    pub gates: bool,
    pub exits: bool,
    /// Only what the game draws while the pod comes down: no teki (see [`comedown_visible`]).
    pub comedown_visible: bool,
    /// Hide objects at a spot in fewer than this share of the group's seeds.
    pub min_share: f64,
    pub scale: f32,
    /// The game the sublevel belongs to, for icons.
    pub game: String,
}

/// An aggregator object id: "t:Name|Carry", "i:Name", "g", "h", "y" or "s".
pub enum AggObject {
    Teki { name: String, carry: Option<String> },
    Treasure(String),
    Gate,
    Hole,
    Geyser,
    Ship,
}

impl AggObject {
    pub fn parse(id: &str) -> AggObject {
        if let Some(rest) = id.strip_prefix("t:") {
            let (name, carry) = match rest.split_once('|') {
                Some((n, c)) if !c.is_empty() => (n.to_string(), Some(c.to_string())),
                Some((n, _)) => (n.to_string(), None),
                None => (rest.to_string(), None),
            };
            return AggObject::Teki { name, carry };
        }
        if let Some(name) = id.strip_prefix("i:") {
            return AggObject::Treasure(name.to_string());
        }
        match id {
            "g" => AggObject::Gate,
            "h" => AggObject::Hole,
            "y" => AggObject::Geyser,
            _ => AggObject::Ship,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            AggObject::Teki { .. } => "teki",
            AggObject::Treasure(_) => "treasure",
            AggObject::Gate => "gate",
            AggObject::Hole => "hole",
            AggObject::Geyser => "geyser",
            AggObject::Ship => "ship",
        }
    }

    pub fn shown(&self, o: &AggOptions) -> bool {
        if o.comedown_visible && !comedown_visible(self.kind()) {
            return false;
        }
        match self {
            AggObject::Treasure(_) => o.treasures,
            AggObject::Teki { carry, .. } => o.teki || (carry.is_some() && o.treasures),
            AggObject::Gate => o.gates,
            _ => o.exits,
        }
    }
}

/// A share of `total` as the aggregator page writes it: "37", "4.2" or "0.3".
pub fn pct(n: u64, total: u64) -> String {
    let p = 100.0 * n as f64 / total.max(1) as f64;
    if p >= 10.0 {
        format!("{}", p.round())
    } else if p >= 1.0 {
        format!("{p:.1}")
    } else {
        // One significant digit, like JavaScript's toPrecision(1).
        format!("{p:.0e}")
            .parse::<f64>()
            .map(|v| format!("{v}"))
            .unwrap_or_else(|_| format!("{p:.2}"))
    }
}

/// Draws one aggregator group, like the aggregator page's map for it.
pub fn render_agg_group(group: &AggGroup, view: [f32; 4], opts: &AggOptions, assets: &Assets) -> Result<Pixmap, String> {
    let cell = CELL * opts.scale;
    let px = |v: f32| v / UNIT * cell;
    // The landing view's trapezoid in the group's frame.
    let landing = group.pod.map(|[x, z, fx, fz]| {
        let [near, far, near_half, far_half] = view;
        let r = (-fz, fx);
        let at = |ahead: f32, side: f32| (x + fx * ahead + r.0 * side, z + fz * ahead + r.1 * side);
        [at(near, -near_half), at(near, near_half), at(far, far_half), at(far, -far_half)]
    });
    // Its extent: units, cells others covered, spots and view, plus half a cell.
    let (mut x0, mut z0, mut x1, mut z1) = (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut grow = |x: f32, z: f32, r: f32| {
        x0 = x0.min(x - r);
        z0 = z0.min(z - r);
        x1 = x1.max(x + r);
        z1 = z1.max(z + r);
    };
    for u in &group.units {
        grow(u.x - u.w * 85.0, u.z - u.h * 85.0, 0.0);
        grow(u.x + u.w * 85.0, u.z + u.h * 85.0, 0.0);
    }
    for &(cx, cz, _) in &group.cells {
        grow(cx * 85.0, cz * 85.0, 85.0);
    }
    for (x, z, _) in &group.spots {
        grow(*x, *z, 85.0);
    }
    for (x, z) in landing.into_iter().flatten() {
        grow(x, z, 0.0);
    }
    if !x0.is_finite() {
        return Err("empty group".into());
    }
    let (x0, z0, x1, z1) = (x0 - 85.0, z0 - 85.0, x1 + 85.0, z1 + 85.0);
    let (w, h) = (px(x1 - x0), px(z1 - z0));
    let mut pixmap = Pixmap::new(w.round().max(1.0) as u32, h.round().max(1.0) as u32).ok_or("the image would be too large")?;
    pixmap.fill(hex("#0f0f0f"));
    let mut cv = Canvas {
        pixmap,
        base: Transform::from_translate(px(-x0), px(-z0)),
        mask: None,
        upright: 0.0,
        alpha: 1.0,
        assets,
    };

    // How often other units covered each cell.
    for &(cx, cz, n) in &group.cells {
        let a = 0.08 + 0.45 * (n as f32 / group.count.max(1) as f32);
        cv.fill_rect(px(cx * 85.0 - 85.0), px(cz * 85.0 - 85.0), cell, cell, rgba(200, 200, 200, a));
    }
    // The group's own map units.
    for u in &group.units {
        match assets.unit(&u.name, &u.game) {
            Some(img) => {
                let (w0, h0) = if u.rot % 2 == 1 { (u.h, u.w) } else { (u.w, u.h) };
                let t = cv
                    .base
                    .pre_translate(px(u.x), px(u.z))
                    .pre_concat(Transform::from_rotate(u.rot as f32 * 90.0))
                    .pre_translate(-w0 * cell / 2.0, -h0 * cell / 2.0)
                    .pre_scale(w0 * cell / img.width() as f32, h0 * cell / img.height() as f32);
                let pp = PixmapPaint {
                    quality: FilterQuality::Nearest,
                    ..PixmapPaint::default()
                };
                cv.pixmap.draw_pixmap(0, 0, img.as_ref().as_ref(), &pp, t, None);
            }
            None => cv.fill_rect(
                px(u.x) - u.w * cell / 2.0,
                px(u.z) - u.h * cell / 2.0,
                u.w * cell,
                u.h * cell,
                hex("#333"),
            ),
        }
    }
    // What the camera sees when you land.
    if let Some(corners) = landing {
        let mut pb = PathBuilder::new();
        for (i, (x, z)) in corners.into_iter().enumerate() {
            if i == 0 {
                pb.move_to(px(x), px(z));
            } else {
                pb.line_to(px(x), px(z));
            }
        }
        pb.close();
        let mut s = stroke((cell / 16.0).max(1.5));
        s.dash = StrokeDash::new(vec![cell / 4.0, cell / 6.0], 0.0);
        cv.stroke_path(pb, hex(LANDING_COLOR), &s);
    }
    // Each spot's objects, stacked, sized by how often they spawned there; likeliest on top.
    let mut spots: Vec<(f32, f32, Vec<(AggObject, u64)>)> = group
        .spots
        .iter()
        .map(|(x, z, objs)| {
            let items = objs
                .iter()
                .map(|(id, n)| (AggObject::parse(id), *n))
                .filter(|(o, n)| o.shown(opts) && *n as f64 / group.count.max(1) as f64 >= opts.min_share)
                .collect::<Vec<_>>();
            (*x, *z, items)
        })
        .filter(|(_, _, items)| !items.is_empty())
        .collect();
    spots.sort_by_key(|(_, _, items)| items[0].1);
    for (sx, sz, items) in &spots {
        let sizes: Vec<f32> = items
            .iter()
            .map(|(_, n)| cell * 0.55 * (0.5 + 0.5 * (*n as f32 / group.count.max(1) as f32)))
            .collect();
        let mut y = px(*sz) - sizes.iter().sum::<f32>() / 2.0;
        let x = px(*sx);
        for ((o, n), size) in items.iter().zip(&sizes) {
            let cy = y + size / 2.0;
            let img = match o {
                AggObject::Teki { name, .. } => assets.teki(name, &opts.game),
                AggObject::Treasure(name) => assets.treasure(name, &opts.game),
                AggObject::Gate => assets.special("Gray_bramble_gate_icon"),
                AggObject::Hole => assets.special("Cave_icon"),
                AggObject::Geyser => assets.special("Geyser_icon"),
                AggObject::Ship => assets.special("pod_icon"),
            };
            if !cv.icon(img, x, cy, *size, None) {
                let color = if matches!(o, AggObject::Treasure(_)) { "#c70" } else { "#7a2a8a" };
                cv.fill_circle(x, cy, size / 2.5, hex(color));
            }
            if let AggObject::Teki { carry: Some(c), .. } = o {
                cv.icon(assets.treasure(c, &opts.game), x + size * 0.4, cy + size * 0.35, size * 0.6, None);
            }
            if cell >= 14.0 {
                let gold = matches!(o, AggObject::Treasure(_) | AggObject::Teki { carry: Some(_), .. });
                let color = if gold { hex("#ffd75a") } else { Color::WHITE };
                cv.label(
                    &format!("{}%", pct(*n, group.count)),
                    x + size / 2.0 + 1.0,
                    cy,
                    color,
                    (cell * 0.24).max(9.0),
                    true,
                );
            }
            y += size;
        }
    }
    Ok(cv.pixmap)
}
