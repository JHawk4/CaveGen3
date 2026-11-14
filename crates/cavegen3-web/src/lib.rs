//! CaveGen3 for the browser.
//!
//! This exposes a small C-style interface so the page needs no JavaScript glue generator:
//!
//! 1. `alloc(len)` reserves `len` bytes; JavaScript writes into them and passes the pointer.
//! 2. `add_files(ptr, len)` adds a bundle of data files (see [`parse_bundle`]). Call it once with
//!    the shared resources and once per game before generating that game's sublevels.
//! 3. `caves()` and `generate(ptr, len, seed)` return a pointer to JSON text, whose length is
//!    `result_len()`. The text stays valid until the next call.
//!
//! The same functions are plain Rust underneath (see [`Web`]), so they're tested natively.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, RwLock},
};

use cavegen3::{
    GameData, Layout, SpawnObject,
    data::FileSource,
};

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
}

impl Default for Web {
    fn default() -> Self {
        Web { source: Box::leak(Box::default()), data: None }
    }
}

impl Web {
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
    pub fn generate_json(&self, sublevel: &str, seed: u32) -> Result<String, String> {
        let data = self.data()?;
        let sublevel = data.sublevel(sublevel).map_err(|e| e.to_string())?;
        let caveinfo = data.caveinfo(&sublevel).map_err(|e| e.to_string())?;
        let num_floors = data.floors(&sublevel.cfg).map_err(|e| e.to_string())?.len();
        Ok(layout_json(&Layout::generate(seed, caveinfo), &sublevel, num_floors, data))
    }
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

fn layout_json(layout: &Layout, sublevel: &cavegen3::Sublevel, num_floors: usize, data: &GameData) -> String {
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
        units.push(format!(
            "{{\"name\":{},\"game\":{},\"x\":{},\"z\":{},\"w\":{},\"h\":{},\"rot\":{},\"type\":{},\"water\":[{}]}}",
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

    let mut objects = Vec::new();
    for (so, pos) in layout.spawn_objects() {
        let (x, z) = (f(pos[0]), f(pos[2]));
        let obj = match so {
            SpawnObject::Teki(info, _) => format!(
                "{{\"kind\":\"teki\",\"name\":{},\"game\":{},\"group\":{},\"falling\":{},\"carrying\":{},\"x\":{x},\"z\":{z}}}",
                q(&info.internal_name),
                q(&info.game),
                info.group,
                info.spawn_method.is_some(),
                info.carrying.as_deref().map_or("null".into(), q),
            ),
            SpawnObject::CapTeki(info, count) => format!(
                "{{\"kind\":\"teki\",\"name\":{},\"game\":{},\"group\":\"cap\",\"falling\":{},\"carrying\":{},\"count\":{count},\"x\":{x},\"z\":{z}}}",
                q(&info.internal_name),
                q(&info.game),
                info.is_falling(),
                info.carrying.as_deref().map_or("null".into(), q),
            ),
            SpawnObject::Item(info) => {
                let value = data.treasure(&info.game, &info.internal_name).map_or(0, |t| t.value);
                format!(
                    "{{\"kind\":\"treasure\",\"name\":{},\"game\":{},\"value\":{value},\"x\":{x},\"z\":{z}}}",
                    q(&info.internal_name),
                    q(&info.game)
                )
            }
            SpawnObject::Gate(info, dir) => {
                format!("{{\"kind\":\"gate\",\"health\":{},\"dir\":{dir},\"x\":{x},\"z\":{z}}}", f(info.health))
            }
            SpawnObject::Hole(plugged) => format!("{{\"kind\":\"hole\",\"plugged\":{plugged},\"x\":{x},\"z\":{z}}}"),
            SpawnObject::Geyser(_) => format!("{{\"kind\":\"geyser\",\"x\":{x},\"z\":{z}}}"),
            SpawnObject::Ship => format!("{{\"kind\":\"ship\",\"x\":{x},\"z\":{z}}}"),
            SpawnObject::Onion(color) => format!("{{\"kind\":\"onion\",\"color\":{color},\"x\":{x},\"z\":{z}}}"),
        };
        objects.push(obj);
    }

    format!(
        "{{\"sublevel\":{},\"long_name\":{},\"game\":{},\"floor\":{},\"floors\":{num_floors},\"seed\":\"{:08X}\",\"width\":{width},\"height\":{height},\"final_floor\":{},\"units\":[{}],\"objects\":[{}]}}",
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

/// The length of the text the last `caves`, `generate` or `add_files` call returned.
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
pub unsafe extern "C" fn generate(ptr: *const u8, len: usize, seed: u32) -> *const u8 {
    let name = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned();
    set_result(with_state(|w| w.generate_json(&name, seed)))
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
        assert!(web.generate_json("SCx6", 0).is_err(), "pikmin2 isn't loaded yet");
        web.add_files(&bundle(root, &["assets/pikmin2"])).unwrap();
        web.add_files(&bundle(root, &["assets/251"])).unwrap();

        // Same layout as generating straight from the data folder.
        let data = GameData::bundled().unwrap();
        let sub = data.sublevel("SCx6").unwrap();
        let expected = Layout::generate(0x1234, data.caveinfo(&sub).unwrap());
        let json = web.generate_json("SCx6", 0x1234).unwrap();
        for u in expected.units.iter() {
            assert!(json.contains(&format!("\"name\":\"{}\",", u.unit.unit_folder_name)));
        }
        assert!(json.contains("\"seed\":\"00001234\""));
        assert!(json.contains("\"kind\":\"ship\""));
        assert!(web.generate_json("251:AT1", 7).unwrap().contains("\"game\":\"251\""));
        assert!(web.caves_json().unwrap().contains("\"name\":\"Colossal Caverns\""));
        assert!(web.generate_json("not a cave", 0).is_err());
    }
}
