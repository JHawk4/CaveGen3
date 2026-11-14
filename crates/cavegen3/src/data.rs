//! Loading the bundled game data: the cave list, caveinfo files and unit files.
//!
//! The cave list format and sublevel naming rules follow Caveripper (MIT,
//! https://github.com/mayabyte/caveripper).

use std::{
    fmt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use encoding_rs::SHIFT_JIS;

use crate::{
    caveinfo::CaveInfo,
    error::{Error, Result},
    parse,
};

/// Somewhere game data files can be read from: a folder, or (later) files built into the binary.
pub trait FileSource: Send + Sync {
    /// Reads a file by its path relative to the data root, using `/` separators.
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

/// Reads data files from a folder on disk.
pub struct DirSource(pub PathBuf);

impl FileSource for DirSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.0.join(path)).ok()
    }
}

/// One cave: a story mode cave or a challenge mode level. Defined in
/// `resources/caveinfo_config.txt`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaveConfig {
    /// `pikmin2` for the vanilla game, or a romhack's folder name.
    pub game: String,
    pub full_name: String,
    pub is_challenge_mode: bool,
    /// The first entry is the canonical short name, e.g. "SCx" or "CH12".
    pub shortened_names: Vec<String>,
    pub caveinfo_filename: String,
}

impl CaveConfig {
    fn parse_list(text: &str) -> Result<Vec<CaveConfig>> {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut fields = line.split(',').map(|f| f.trim().to_string());
                let mut next = || fields.next().ok_or_else(|| Error::new(format!("incomplete cave entry: {line}")));
                let game = next()?;
                let full_name = next()?;
                let is_challenge_mode = next()?.parse().map_err(|_| Error::new(format!("bad challenge mode flag: {line}")))?;
                let caveinfo_filename = next()?;
                let shortened_names = fields.filter(|f| !f.is_empty()).collect::<Vec<_>>();
                if shortened_names.is_empty() {
                    return Err(Error::new(format!("cave has no short name: {line}")));
                }
                Ok(CaveConfig { game, full_name, is_challenge_mode, shortened_names, caveinfo_filename })
            })
            .collect()
    }

    pub fn is_colossal_caverns(&self) -> bool {
        self.full_name.eq_ignore_ascii_case("Colossal Caverns")
    }

    /// The canonical short name, e.g. "SCx".
    pub fn short_name(&self) -> &str {
        &self.shortened_names[0]
    }
}

/// A specific floor of a specific cave.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sublevel {
    pub cfg: CaveConfig,
    /// 1-indexed.
    pub floor: usize,
}

impl Sublevel {
    /// The name in CaveGen's format, e.g. "SCx-3" or "CH12-1".
    pub fn normalized_name(&self) -> String {
        format!("{}-{}", self.cfg.short_name(), self.floor)
    }

    /// e.g. "SCx3"; challenge mode levels use the normalized name.
    pub fn short_name(&self) -> String {
        if self.cfg.is_challenge_mode { self.normalized_name() } else { format!("{}{}", self.cfg.short_name(), self.floor) }
    }

    /// e.g. "Submerged Castle 3".
    pub fn long_name(&self) -> String {
        format!("{} {}", self.cfg.full_name, self.floor)
    }
}

impl fmt::Display for Sublevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.short_name())
    }
}

/// All game data CaveGen3 knows about, with parsed caveinfo cached per cave.
///
/// Parsing happens on first use and is safe to share between threads.
pub struct GameData {
    source: Box<dyn FileSource>,
    caves: Vec<CaveConfig>,
    parsed: Vec<OnceLock<Result<Vec<CaveInfo>>>>,
}

impl GameData {
    /// The data folder bundled with CaveGen3. Set `CAVEGEN3_DATA` to use another folder.
    pub fn bundled() -> Result<GameData> {
        match std::env::var_os("CAVEGEN3_DATA") {
            Some(dir) => GameData::from_dir(dir),
            None => GameData::from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")),
        }
    }

    /// Loads data from a folder holding `assets/<game>` and `resources`.
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<GameData> {
        GameData::from_source(Box::new(DirSource(dir.as_ref().to_path_buf())))
    }

    pub fn from_source(source: Box<dyn FileSource>) -> Result<GameData> {
        let list = source
            .read("resources/caveinfo_config.txt")
            .ok_or_else(|| Error::new("missing resources/caveinfo_config.txt"))?;
        let caves = CaveConfig::parse_list(&String::from_utf8_lossy(&list))?;
        let parsed = caves.iter().map(|_| OnceLock::new()).collect();
        Ok(GameData { source, caves, parsed })
    }

    /// Every known cave, including romhacks whose data isn't bundled.
    pub fn caves(&self) -> &[CaveConfig] {
        &self.caves
    }

    /// Reads a data file as raw bytes.
    pub fn read_raw(&self, path: &str) -> Result<Vec<u8>> {
        self.source.read(path).ok_or_else(|| Error::new(format!("missing data file {path}")))
    }

    /// Reads a text file. Game files (under `assets/`) are Shift-JIS; the rest are UTF-8.
    pub fn read_text(&self, path: &str) -> Result<String> {
        let bytes = self.read_raw(path)?;
        if path.starts_with("assets/") {
            Ok(SHIFT_JIS.decode(&bytes).0.into_owned())
        } else {
            String::from_utf8(bytes).map_err(|_| Error::new(format!("{path} isn't valid UTF-8")))
        }
    }

    /// Whether this data has the files for a game (e.g. `pikmin2`, `colossal`).
    pub fn has_game(&self, game: &str) -> bool {
        self.caves
            .iter()
            .filter(|c| c.game.eq_ignore_ascii_case(game))
            .any(|c| self.source.read(&format!("assets/{}/caveinfo/{}", c.game, c.caveinfo_filename)).is_some())
    }

    /// Finds a cave by any of its names. `game` restricts the search to one game;
    /// `challenge_only` skips story mode caves.
    pub fn find_cave(&self, name: &str, game: Option<&str>, challenge_only: bool) -> Option<&CaveConfig> {
        self.caves.iter().find(|cfg| {
            game.is_none_or(|g| cfg.game.eq_ignore_ascii_case(g))
                && (!challenge_only || cfg.is_challenge_mode)
                && (cfg.full_name.eq_ignore_ascii_case(name) || cfg.shortened_names.iter().any(|n| n.eq_ignore_ascii_case(name)))
        })
    }

    /// All floors of a cave, parsed on first use.
    pub fn floors(&self, cfg: &CaveConfig) -> Result<&[CaveInfo]> {
        let idx = self
            .caves
            .iter()
            .position(|c| c == cfg)
            .ok_or_else(|| Error::new(format!("unknown cave {}", cfg.full_name)))?;
        self.parsed[idx]
            .get_or_init(|| parse::parse_cave(cfg, self).map_err(|e| e.context(format!("{} ({})", cfg.full_name, cfg.game))))
            .as_deref()
            .map_err(Clone::clone)
    }

    /// The caveinfo for one sublevel.
    pub fn caveinfo(&self, sublevel: &Sublevel) -> Result<&CaveInfo> {
        self.floors(&sublevel.cfg)?
            .get(sublevel.floor.wrapping_sub(1))
            .ok_or_else(|| Error::new(format!("{} has no floor {}", sublevel.cfg.full_name, sublevel.floor)))
    }

    /// Parses a sublevel name such as "SCx6", "scx-6", "Snagret Hole 6", "CH12-1",
    /// "ch-cm2" (challenge mode by short name), "colossal", or "newyear:BG3".
    pub fn sublevel(&self, input: &str) -> Result<Sublevel> {
        let unknown = || Error::new(format!("unrecognized sublevel \"{input}\""));
        let (game, rest) = match input.split_once(':') {
            Some((game, rest)) => (Some(game.trim().to_ascii_lowercase()), rest.to_ascii_lowercase()),
            None => (None, input.to_ascii_lowercase()),
        };
        let game = game.as_deref();
        let components: Vec<&str> = rest.split('-').map(str::trim).filter(|c| !c.is_empty()).collect();
        let lookup = |name: &str, challenge_only: bool, floor: usize| {
            self.find_cave(name, game, challenge_only).map(|cfg| Sublevel { cfg: cfg.clone(), floor }).ok_or_else(unknown)
        };
        match components.as_slice() {
            [only] if only.eq_ignore_ascii_case("colossal") => lookup("colossal", false, 1),
            [only] => {
                let (name, floor) = split_short_name(only).ok_or_else(unknown)?;
                lookup(name, false, floor)
            }
            [prefix, name] if *prefix == "ch" || *prefix == "cm" => {
                let (name, floor) = split_short_name(name).ok_or_else(unknown)?;
                lookup(name, true, floor)
            }
            [name, floor] => lookup(name, false, floor.parse().map_err(|_| unknown())?),
            _ => Err(unknown()),
        }
    }
}

/// Splits "scx6" or "snagret hole 6" into the first run of letters (plus spaces, `'` and `_`)
/// and the first run of digits.
fn split_short_name(input: &str) -> Option<(&str, usize)> {
    let is_word = |c: char| c.is_alphabetic() || c.is_whitespace() || c == '\'' || c == '_';
    let start = input.find(is_word)?;
    let end = input[start..].find(|c: char| !is_word(c)).map_or(input.len(), |i| start + i);
    let name = input[start..end].trim();
    let digits_start = input.find(|c: char| c.is_ascii_digit())?;
    let digits_end = input[digits_start..].find(|c: char| !c.is_ascii_digit()).map_or(input.len(), |i| digits_start + i);
    let floor = input[digits_start..digits_end].parse().ok()?;
    (!name.is_empty()).then_some((name, floor))
}
