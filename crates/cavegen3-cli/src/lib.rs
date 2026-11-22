//! CaveGen3's command-line tool: everything the web page does, from a terminal. Layouts and
//! aggregator groups are written as PNG images; CaveInfo, judges, searches, stats and counts are
//! printed as text (or JSON).

pub mod render;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cavegen3::{GameData, Generator};
use cavegen3_web::{DETAILS, SWITCH, Web};
use render::{Assets, Layout, Options};
use serde_json::Value;

/// The data folder: `CAVEGEN3_DATA` if set, otherwise the repository's `data` folder.
pub fn default_data_dir() -> PathBuf {
    match std::env::var_os("CAVEGEN3_DATA") {
        Some(dir) => dir.into(),
        None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
    }
}

/// Generates layouts, runs the generator's commands, and draws.
pub struct Painter {
    pub web: Web,
    pub assets: Assets,
    /// Which `rand()` layouts are generated with (GC unless set).
    pub generator: Generator,
    names: Names,
}

impl Painter {
    pub fn new(data_dir: &Path) -> Result<Painter, String> {
        let data = GameData::from_dir(data_dir).map_err(|e| e.to_string())?;
        Ok(Painter {
            web: Web::from_data(data),
            assets: Assets::new(data_dir)?,
            generator: Generator::Gc,
            names: Names::load(data_dir),
        })
    }

    /// One layout, as the web page gets it.
    /// `seed` has 64 bits with the Switch generator, 32 otherwise.
    pub fn layout(&self, sublevel: &str, seed: u64, details: bool) -> Result<Layout, String> {
        let mut flags = if details { DETAILS } else { 0 };
        if self.generator == Generator::Switch {
            flags |= SWITCH;
        }
        let json = self.web.generate_json(sublevel, seed, flags)?;
        serde_json::from_str(&json).map_err(|e| format!("bad layout JSON: {e}"))
    }

    /// One layout's image, encoded as PNG.
    pub fn png(&self, sublevel: &str, seed: u64, opts: &Options) -> Result<(Layout, Vec<u8>), String> {
        let layout = self.layout(sublevel, seed, opts.needs_details())?;
        let pixmap = render::render(&layout, opts, &self.assets)?;
        let png = pixmap.encode_png().map_err(|e| e.to_string())?;
        Ok((layout, png))
    }

    /// One of the generator's tab-separated commands (`judge`, `search`, `caveinfo`), as JSON.
    pub fn query(&self, args: &[&str]) -> Result<Value, String> {
        let json = self.web.query(&args.join("\t"))?;
        serde_json::from_str(&json).map_err(|e| format!("bad JSON from {}: {e}", args[0]))
    }

    /// The in-game English name of a teki or treasure, or its internal name.
    pub fn name(&self, kind: &str, internal: &str) -> String {
        self.names.get(kind, internal).unwrap_or_else(|| internal.to_string())
    }

    /// The name, then the internal name in brackets when they differ.
    pub fn name_and_id(&self, kind: &str, internal: &str) -> String {
        match self.names.get(kind, internal) {
            Some(n) if !n.eq_ignore_ascii_case(internal) => format!("{n} ({internal})"),
            _ => internal.to_string(),
        }
    }

    /// Which map units look the same turned (1: a quarter turn, 2: a half turn, 4: never), from
    /// their radar images, as the aggregator page works it out: "name=period;...".
    pub fn image_periods(&self, sublevel: &str) -> Result<String, String> {
        let ci = self.query(&["caveinfo", sublevel])?;
        let mut out = Vec::new();
        for u in ci["units"].as_array().into_iter().flatten() {
            let (Some(name), Some(game)) = (u["name"].as_str(), u["game"].as_str()) else {
                continue;
            };
            let Some(img) = self.assets.unit(name, game) else { continue };
            let (w, h) = (img.width() as usize, img.height() as usize);
            let alpha: Vec<u8> = img.pixels().iter().map(|p| p.alpha()).collect();
            let v = |x: usize, y: usize| alpha[y * w + x] as i32;
            let mismatch = |map: &dyn Fn(usize, usize) -> (usize, usize)| {
                let mut bad = 0;
                for y in 0..h {
                    for x in 0..w {
                        let (x2, y2) = map(x, y);
                        if (v(x, y) - v(x2, y2)).abs() > 40 {
                            bad += 1;
                        }
                    }
                }
                bad as f64 / (w * h).max(1) as f64
            };
            let period = if w == h && mismatch(&|x, y| (y, w - 1 - x)) < 0.03 {
                1
            } else if mismatch(&|x, y| (w - 1 - x, h - 1 - y)) < 0.03 {
                2
            } else {
                4
            };
            out.push(format!("{name}={period}"));
        }
        Ok(out.join(";"))
    }
}

/// English names from `resources/names.json`.
struct Names(HashMap<String, HashMap<String, String>>);

impl Names {
    fn load(data_dir: &Path) -> Names {
        let mut out = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(data_dir.join("resources/names.json"))
            && let Ok(Value::Object(kinds)) = serde_json::from_str::<Value>(&text)
        {
            for (kind, entries) in kinds {
                let map = entries
                    .as_object()
                    .map(|m| {
                        m.iter()
                            .filter_map(|(k, v)| Some((k.to_ascii_lowercase(), v["en"].as_str()?.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                out.insert(kind, map);
            }
        }
        Names(out)
    }

    fn get(&self, kind: &str, internal: &str) -> Option<String> {
        self.0.get(kind)?.get(&internal.to_ascii_lowercase()).cloned()
    }
}
