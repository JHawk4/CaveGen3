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

    /// Lists a folder: each entry's name and whether it's a folder itself.
    fn list(&self, dir: &str) -> Vec<(String, bool)>;
}

/// Reads data files from a folder on disk.
///
/// Like the game's file system, names are matched ignoring case: some romhacks refer to a file
/// with different capitalization than it has, which only matters on case-sensitive systems.
pub struct DirSource(pub PathBuf);

impl FileSource for DirSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.0.join(path)).ok().or_else(|| {
            let mut dir = self.0.clone();
            for part in path.split('/') {
                let entry = std::fs::read_dir(&dir).ok()?.filter_map(|e| e.ok()).find(|e| e.file_name().eq_ignore_ascii_case(part))?;
                dir = entry.path();
            }
            std::fs::read(dir).ok()
        })
    }

    fn list(&self, dir: &str) -> Vec<(String, bool)> {
        let Ok(entries) = std::fs::read_dir(self.0.join(dir)) else { return Vec::new() };
        let mut out: Vec<(String, bool)> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| Some((e.file_name().into_string().ok()?, e.path().is_dir())))
            .collect();
        out.sort();
        out
    }
}

/// A treasure's carry weights and value, from `otakara_config.txt` and `item_config.txt`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Treasure {
    pub internal_name: String,
    pub game: String,
    pub min_carry: u32,
    pub max_carry: u32,
    pub value: u32,
    /// Its bounding radius, which the treasure gauge subtracts from the distance.
    pub radius: f32,
    /// Its height and how deep it's buried: `depth >= height` means fully buried (as CaveGen).
    pub height: f32,
    pub depth: f32,
}

impl Treasure {
    /// Whether it starts fully under the ground, as CaveGen's -drawNoBuriedItems decides.
    pub fn fully_buried(&self) -> bool {
        self.depth > 0.0 && self.depth >= self.height
    }
}

/// Parses a treasure config file: `{ ... }` blocks of `key value` lines, using the
/// `name`, `min`, `max` and `money` keys.
fn parse_treasure_config(text: &str, game: &str) -> Vec<Treasure> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        let block = &after[..close];
        rest = &after[close + 1..];
        let mut fields = std::collections::HashMap::new();
        for line in block.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            if let (Some(first), Some(last)) = (words.first(), words.last()) {
                fields.insert(*first, *last);
            }
        }
        let field = |k: &str| fields.get(k).and_then(|v| v.parse().ok());
        let float = |k: &str| fields.get(k).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
        if let (Some(name), Some(min), Some(max), Some(money)) = (fields.get("name"), field("min"), field("max"), field("money")) {
            out.push(Treasure {
                internal_name: name.to_string(),
                game: game.to_string(),
                min_carry: min,
                max_carry: max,
                value: money,
                radius: float("radius"),
                height: float("height"),
                depth: float("depth"),
            });
        }
    }
    out
}

/// Names of everything in the bundled data, for telling what a query refers to.
#[derive(Debug, Default)]
pub struct Catalog {
    /// Lowercase teki names (icon names, plus eggs, bomb rocks and fire geysers).
    pub teki: Vec<String>,
    /// Lowercase map unit folder names.
    pub units: Vec<String>,
    pub treasures: Vec<Treasure>,
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

/// Colossal Caverns' generation options, as in CaveGen: `-ultraRandomizer` (shuffle the map
/// unit queues far more), `-extraUnits` (Pikmin 251's units too) and `-5onion` (five Onions).
/// In a sublevel name they're suffixes: "colossal+ultra+extra+5onion".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Variant {
    pub ultra_randomizer: bool,
    pub extra_units: bool,
    pub five_onions: bool,
}

impl Variant {
    pub fn is_default(self) -> bool {
        self == Variant::default()
    }

    /// An index 0-7, for caching each combination.
    fn index(self) -> usize {
        self.ultra_randomizer as usize | (self.extra_units as usize) << 1 | (self.five_onions as usize) << 2
    }

    /// The name suffix, e.g. "+ultra+5onion", or "" for the default.
    pub fn suffix(self) -> String {
        let mut s = String::new();
        if self.ultra_randomizer {
            s.push_str("+ultra");
        }
        if self.extra_units {
            s.push_str("+extra");
        }
        if self.five_onions {
            s.push_str("+5onion");
        }
        s
    }

    /// Reads one suffix word; false if it isn't one.
    fn set(&mut self, word: &str) -> bool {
        match word.trim().to_ascii_lowercase().as_str() {
            "ultra" | "ultrarandomizer" => self.ultra_randomizer = true,
            "extra" | "extraunits" => self.extra_units = true,
            "5onion" | "5onions" | "fiveonions" => self.five_onions = true,
            _ => return false,
        }
        true
    }
}

/// A specific floor of a specific cave.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sublevel {
    pub cfg: CaveConfig,
    /// 1-indexed.
    pub floor: usize,
    /// Colossal Caverns' options; always the default elsewhere.
    pub variant: Variant,
}

impl Sublevel {
    /// The name in CaveGen's format, e.g. "SCx-3" or "CH12-1".
    pub fn normalized_name(&self) -> String {
        format!("{}-{}{}", self.cfg.short_name(), self.floor, self.variant.suffix())
    }

    /// e.g. "SCx3"; challenge mode levels use the normalized name.
    pub fn short_name(&self) -> String {
        if self.cfg.is_challenge_mode {
            self.normalized_name()
        } else {
            format!("{}{}{}", self.cfg.short_name(), self.floor, self.variant.suffix())
        }
    }

    /// e.g. "Submerged Castle 3".
    pub fn long_name(&self) -> String {
        format!("{} {}{}", self.cfg.full_name, self.floor, self.variant.suffix())
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
    /// Per cave, per [`Variant`] combination.
    parsed: Vec<[OnceLock<Result<Vec<CaveInfo>>>; 8]>,
    catalog: OnceLock<Catalog>,
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
        let parsed = caves.iter().map(|_| std::array::from_fn(|_| OnceLock::new())).collect();
        Ok(GameData { source, caves, parsed, catalog: OnceLock::new() })
    }

    /// Games with data in `assets/`.
    pub fn games(&self) -> Vec<String> {
        self.source.list("assets").into_iter().filter(|(_, dir)| *dir).map(|(name, _)| name).collect()
    }

    /// Every known teki, map unit and treasure across the bundled games.
    pub fn catalog(&self) -> &Catalog {
        self.catalog.get_or_init(|| {
            let mut catalog = Catalog::default();
            for game in self.games() {
                // Eggs, bomb rocks and fire geysers have no icon in the teki folder.
                catalog.teki.extend(["egg", "bomb", "hiba"].map(String::from));
                for (file, _) in self.source.list(&format!("assets/{game}/teki")) {
                    if let Some(name) = file.strip_suffix(".png") {
                        catalog.teki.push(name.to_ascii_lowercase());
                    }
                }
                for (dir, is_dir) in self.source.list(&format!("assets/{game}/mapunits")) {
                    if is_dir {
                        catalog.units.push(dir.to_ascii_lowercase());
                    }
                }
                for file in ["otakara_config.txt", "item_config.txt"] {
                    if let Ok(text) = self.read_text(&format!("assets/{game}/{file}")) {
                        catalog.treasures.extend(parse_treasure_config(&text, &game));
                    }
                }
            }
            catalog
        })
    }

    /// A treasure's info. Romhacks fall back to vanilla treasures, as 216 needs.
    pub fn treasure(&self, game: &str, name: &str) -> Option<&Treasure> {
        let find = |g: &str| self.catalog().treasures.iter().find(|t| t.game == g && t.internal_name.eq_ignore_ascii_case(name));
        find(game).or_else(|| if game != "pikmin2" { find("pikmin2") } else { None })
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

    /// Finds a cave by any of its names. `game` restricts the search to one game; without one,
    /// only vanilla caves are searched, since romhacks reuse vanilla abbreviations (New Year's
    /// Elemental River is "ER", also Emperor's Realm's). Colossal Caverns counts as vanilla.
    /// `challenge_only` skips story mode caves.
    pub fn find_cave(&self, name: &str, game: Option<&str>, challenge_only: bool) -> Option<&CaveConfig> {
        let in_game = |cfg: &CaveConfig| match game {
            Some(g) => cfg.game.eq_ignore_ascii_case(g),
            None => cfg.game == "pikmin2" || cfg.game == "colossal",
        };
        self.caves.iter().find(|cfg| {
            in_game(cfg)
                && (!challenge_only || cfg.is_challenge_mode)
                && (cfg.full_name.eq_ignore_ascii_case(name) || cfg.shortened_names.iter().any(|n| n.eq_ignore_ascii_case(name)))
        })
    }

    /// All floors of a cave, parsed on first use.
    pub fn floors(&self, cfg: &CaveConfig) -> Result<&[CaveInfo]> {
        self.floors_variant(cfg, Variant::default())
    }

    /// All floors of a cave with Colossal Caverns' options, parsed on first use.
    pub fn floors_variant(&self, cfg: &CaveConfig, variant: Variant) -> Result<&[CaveInfo]> {
        let idx = self
            .caves
            .iter()
            .position(|c| c == cfg)
            .ok_or_else(|| Error::new(format!("unknown cave {}", cfg.full_name)))?;
        self.parsed[idx][variant.index()]
            .get_or_init(|| parse::parse_cave(cfg, self, variant).map_err(|e| e.context(format!("{} ({})", cfg.full_name, cfg.game))))
            .as_deref()
            .map_err(Clone::clone)
    }

    /// The caveinfo for one sublevel.
    pub fn caveinfo(&self, sublevel: &Sublevel) -> Result<&CaveInfo> {
        self.floors_variant(&sublevel.cfg, sublevel.variant)?
            .get(sublevel.floor.wrapping_sub(1))
            .ok_or_else(|| Error::new(format!("{} has no floor {}", sublevel.cfg.full_name, sublevel.floor)))
    }

    /// Parses a sublevel name such as "SCx6", "scx-6", "Snagret Hole 6", "CH12-1",
    /// "ch-cm2" (challenge mode by short name), "colossal", or "newyear:BG3". Names without a
    /// `game:` prefix are vanilla Pikmin 2 caves.
    pub fn sublevel(&self, input: &str) -> Result<Sublevel> {
        self.sublevel_in(input, None)
    }

    /// Like [`GameData::sublevel`], but a name without a `game:` prefix belongs to
    /// `default_game` when one is given (vanilla otherwise).
    pub fn sublevel_in(&self, input: &str, default_game: Option<&str>) -> Result<Sublevel> {
        // Colossal Caverns' options come after the name: "colossal+ultra+5onion".
        let mut parts = input.split('+');
        let name = parts.next().unwrap_or_default();
        let mut variant = Variant::default();
        for word in parts {
            if !variant.set(word) {
                return Err(Error::new(format!("unknown option \"+{word}\" in \"{input}\": the options are +ultra, +extra and +5onion")));
            }
        }
        let mut sublevel = self.sublevel_plain(name, default_game)?;
        if !variant.is_default() && !sublevel.cfg.is_colossal_caverns() {
            return Err(Error::new(format!("\"{input}\": only Colossal Caverns has the +ultra, +extra and +5onion options")));
        }
        sublevel.variant = variant;
        Ok(sublevel)
    }

    fn sublevel_plain(&self, input: &str, default_game: Option<&str>) -> Result<Sublevel> {
        let unknown = || Error::new(format!("unrecognized sublevel \"{input}\""));
        let (game, rest) = match input.split_once(':') {
            Some((game, rest)) => (Some(game.trim().to_ascii_lowercase()), rest.to_ascii_lowercase()),
            None => (default_game.map(str::to_ascii_lowercase), input.to_ascii_lowercase()),
        };
        let game = game.as_deref();
        let components: Vec<&str> = rest.split('-').map(str::trim).filter(|c| !c.is_empty()).collect();
        let lookup = |name: &str, challenge_only: bool, floor: usize| {
            self.find_cave(name, game, challenge_only).map(|cfg| Sublevel { cfg: cfg.clone(), floor, variant: Variant::default() }).ok_or_else(unknown)
        };
        match components.as_slice() {
            [only] if only.eq_ignore_ascii_case("colossal") => lookup("colossal", false, 1),
            [only] => {
                let (name, floor) = split_short_name(only).ok_or_else(unknown)?;
                lookup(name, false, floor)
            }
            // "CH-BB2". (New Year's "CH-5" is Commodore House 5, handled below.)
            [prefix, name] if (*prefix == "ch" || *prefix == "cm") && split_short_name(name).is_some() => {
                let (name, floor) = split_short_name(name).ok_or_else(unknown)?;
                lookup(name, true, floor)
            }
            [name, floor] => {
                // "CH12-2", or the "CH12-CM2" form that `CaveInfo::name` produces.
                let floor = match floor.parse() {
                    Ok(f) => f,
                    Err(_) => split_short_name(floor).ok_or_else(unknown)?.1,
                };
                lookup(name, false, floor)
            }
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
