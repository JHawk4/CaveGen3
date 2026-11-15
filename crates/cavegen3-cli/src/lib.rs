//! CaveGen3's command-line tool: generates layouts and writes them as PNG images.

pub mod render;

use std::path::{Path, PathBuf};

use cavegen3::GameData;
use cavegen3_web::{DETAILS, Web};
use render::{Assets, Layout, Options};

/// The data folder: `CAVEGEN3_DATA` if set, otherwise the repository's `data` folder.
pub fn default_data_dir() -> PathBuf {
    match std::env::var_os("CAVEGEN3_DATA") {
        Some(dir) => dir.into(),
        None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
    }
}

/// Generates layouts and draws them.
pub struct Painter {
    pub web: Web,
    pub assets: Assets,
}

impl Painter {
    pub fn new(data_dir: &Path) -> Result<Painter, String> {
        let data = GameData::from_dir(data_dir).map_err(|e| e.to_string())?;
        Ok(Painter {
            web: Web::from_data(data),
            assets: Assets::new(data_dir)?,
        })
    }

    /// One layout, as the web page gets it.
    pub fn layout(&self, sublevel: &str, seed: u32, details: bool) -> Result<Layout, String> {
        let json = self.web.generate_json(sublevel, seed, if details { DETAILS } else { 0 })?;
        serde_json::from_str(&json).map_err(|e| format!("bad layout JSON: {e}"))
    }

    /// One layout's image, encoded as PNG.
    pub fn png(&self, sublevel: &str, seed: u32, opts: &Options) -> Result<(Layout, Vec<u8>), String> {
        let layout = self.layout(sublevel, seed, opts.needs_details())?;
        let pixmap = render::render(&layout, opts, &self.assets)?;
        let png = pixmap.encode_png().map_err(|e| e.to_string())?;
        Ok((layout, png))
    }
}
