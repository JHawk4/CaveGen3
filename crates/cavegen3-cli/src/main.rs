use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use cavegen3::{Generator, rng::Rng};
use cavegen3_cli::{
    Painter, default_data_dir,
    render::{self, AggOptions, AggResult, OPTION_NAMES, Options, pct},
};
use cavegen3_web::{mix64, seed_at, seed_in};
use clap::{Args, Parser, Subcommand};
use rayon::prelude::*;
use serde_json::Value;

/// CaveGen3 on the command line: everything the web page does. Layouts and aggregator groups
/// are saved as PNG images; the rest prints text.
///
/// With no command, the arguments are for `generate`:
///   cavegen3 SCx-6 1234ABCD
///   cavegen3 BK-4 -n 100 --seeds r -o bk4
///   cavegen3 SCx-6 1234ABCD --landing --screen 4:3
///   cavegen3 SCx-6 1234ABCD --view louie
///   cavegen3 EC-1 1234ABCD --show landingBox,comedownBox,podAngle --screen 4:3
///   cavegen3 FC-3 0 --show spawnpoints,doorScores,quickGlance --hide water
///   cavegen3 colossal+ultra+5onion 0
///   cavegen3 EC-1 306306F66F0443C4 --rng switch      (Switch: 64-bit seeds)
///
/// Other commands:
///   cavegen3 caveinfo FC-3
///   cavegen3 judge BK-4 pod 1234ABCD
///   cavegen3 search BK-4 --judge pod -n 100000 --top 10
///   cavegen3 search SCx-7 --query "minihoudai < 2" -n 50000
///   cavegen3 stats SR-5 "BlackPom = 1" -n 20000
///   cavegen3 count SCx-7 minihoudai -n 20000
///   cavegen3 aggregate HoB-2 -n 20000 -o agg
///   cavegen3 caves 251
#[derive(Parser)]
#[command(
    name = "cavegen3",
    version,
    verbatim_doc_comment,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    #[command(flatten)]
    generate: GenerateArgs,

    /// The data folder (default: CAVEGEN3_DATA, else the repository's data folder).
    #[arg(long, global = true)]
    data: Option<PathBuf>,

    /// The game's random number generator: gc (GameCube, Wii, Wii U) or switch (Pikmin 2 HD on
    /// Switch, 64-bit seeds). Switch works for drawing layouts and judge.
    #[arg(long, global = true, default_value = "gc")]
    rng: Generator,
}

#[derive(Subcommand)]
enum Cmd {
    /// Draw layouts as PNG images (the default command).
    Generate(GenerateArgs),
    /// List the caves and their sublevel names, for one game or all.
    Caves {
        /// pikmin2, 251, newyear, colossal, 216 or bikmin.
        game: Option<String>,
    },
    /// What a sublevel can generate: its limits, teki, treasures, gates, alcove teki and units.
    Caveinfo {
        #[arg(help = SUBLEVEL_HELP)]
        sublevel: String,
        /// Print the generator's JSON instead.
        #[arg(long)]
        json: bool,
    },
    /// Score seeds with one of CaveGen's judges (lower is better).
    Judge {
        #[arg(help = SUBLEVEL_HELP)]
        sublevel: String,
        #[arg(help = JUDGE_HELP)]
        judge: String,
        /// One or more seeds in hex.
        #[arg(required = true)]
        seeds: Vec<String>,
    },
    /// Test many seeds: list the best by a judge or the ones matching a query, with the share
    /// that matched and, with --count, how many of something each seed has.
    Search(SearchArgs),
    /// The share of seeds matching a query (Caveripper's stats).
    Stats {
        #[arg(help = SUBLEVEL_HELP)]
        sublevel: String,
        /// A Caveripper-style query, e.g. "minihoudai < 2" (the sublevel can be left out).
        query: String,
        #[command(flatten)]
        range: Range,
    },
    /// How many of something each seed has, as a distribution (CaveGen's -count).
    Count {
        #[arg(help = SUBLEVEL_HELP)]
        sublevel: String,
        /// A teki or treasure internal name, or teki, treasure, gate or onion.
        what: String,
        #[command(flatten)]
        range: Range,
    },
    /// CaveGen's aggregator: group seeds that look alike and draw where things spawn in each.
    Aggregate(AggArgs),
    /// List the map options for --show and --hide.
    Options,
}

const SUBLEVEL_HELP: &str = "The sublevel, like SCx-6, BK-1 or \"Subterranean Complex 6\". Romhacks take a game \
    prefix (251:AT-2, 216:CC-1); Colossal Caverns is \"colossal\", with options +ultra, +extra and +5onion.";
const JUDGE_HELP: &str = "A judge: score, pod, at, attk, cmat, key, colossal or mapunitcount.";

#[derive(Args)]
struct GenerateArgs {
    #[arg(help = SUBLEVEL_HELP, required_unless_present = "list_options")]
    sublevel: Option<String>,

    /// The seed in hex (up to 8 digits, 16 with --rng switch; 0x optional). A random one if left out.
    seed: Option<String>,

    /// How many layouts to draw.
    #[arg(short = 'n', long, default_value_t = 1)]
    count: u64,

    /// How seeds follow the first: c counts up by one, g steps the game's RNG (CaveGen's
    /// -consecutiveSeeds), r picks spread-out random seeds.
    #[arg(long, default_value = "c", value_parser = ["c", "g", "r"])]
    seeds: String,

    /// The folder for the images. Each is named <sublevel>_<SEED>.png.
    #[arg(short, long, default_value = "output")]
    out: PathBuf,

    #[command(flatten)]
    look: LookArgs,

    /// List the map options and exit.
    #[arg(long)]
    list_options: bool,
}

/// How layouts are drawn.
#[derive(Args, Clone)]
struct LookArgs {
    /// Image size: 1 draws a map grid cell as 40 pixels.
    #[arg(long, default_value_t = 1.0)]
    scale: f32,

    /// Draw only what one camera shows, turned as on screen: pod (when you land, after pressing
    /// L), olimar or louie (when you land, without L, playing as that captain), or comedown
    /// (while the sublevel's name is on screen).
    #[arg(long, value_parser = ["pod", "olimar", "louie", "comedown"])]
    view: Option<String>,

    /// The same as --view pod. (--comedown is its old name.)
    #[arg(long, alias = "comedown", conflicts_with = "view")]
    landing: bool,

    /// The game's screen shape, for the landing and comedown views: 16:9 or 4:3.
    #[arg(long, default_value = "16:9", value_parser = ["16:9", "4:3"])]
    screen: String,

    /// Map options to turn on, comma-separated (see `cavegen3 options`).
    #[arg(long, value_delimiter = ',')]
    show: Vec<String>,

    /// Map options to turn off, comma-separated.
    #[arg(long, value_delimiter = ',')]
    hide: Vec<String>,
}

impl LookArgs {
    fn options(&self) -> Result<Options, String> {
        if !(0.05..=20.0).contains(&self.scale) {
            return Err("--scale must be between 0.05 and 20".into());
        }
        let mut opts = Options {
            view: match (&self.view, self.landing) {
                (Some(v), _) => Some(render::View::parse(v)?),
                (None, true) => Some(render::View::Landing(render::LandingCamera::Pod)),
                (None, false) => None,
            },
            scale: self.scale,
            ..Options::default()
        };
        if self.screen == "4:3" {
            opts.aspect = 4.0 / 3.0;
        }
        for name in &self.show {
            opts.set(name, true)?;
        }
        for name in &self.hide {
            opts.set(name, false)?;
        }
        Ok(opts)
    }
}

/// Which seeds to test.
#[derive(Args, Clone)]
struct Range {
    /// How many seeds.
    #[arg(short = 'n', long = "num", default_value_t = 10000)]
    num: u64,

    /// r: random seeds; c: counting up from --start; g: the game's RNG stepped from --start.
    #[arg(long, default_value = "r", value_parser = ["c", "g", "r"])]
    seeds: String,

    /// The first seed in hex, for --seeds c or g (default 0).
    #[arg(long)]
    start: Option<String>,
}

impl Range {
    fn start(&self) -> Result<u64, String> {
        match (&self.start, self.seeds.as_str()) {
            (Some(s), _) => Ok(parse_seed(s)? as u64),
            (None, "r") => Ok(random_seed() as u64),
            (None, _) => Ok(0),
        }
    }
}

#[derive(Args)]
struct SearchArgs {
    #[arg(help = SUBLEVEL_HELP)]
    sublevel: String,

    /// Rank seeds by this judge (lower is better).
    #[arg(long, help = JUDGE_HELP)]
    judge: Option<String>,

    /// Keep only seeds matching a Caveripper-style query (the sublevel can be left out).
    #[arg(long)]
    query: Option<String>,

    /// Also tally how many of something each seed has (CaveGen's -count).
    #[arg(long)]
    count: Option<String>,

    #[command(flatten)]
    range: Range,

    /// How many seeds to list.
    #[arg(long, default_value_t = 20)]
    top: usize,

    /// Also draw the listed seeds into this folder.
    #[arg(long)]
    images: Option<PathBuf>,

    #[command(flatten)]
    look: LookArgs,

    /// Print JSON instead of text.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AggArgs {
    #[arg(help = SUBLEVEL_HELP)]
    sublevel: String,

    /// How to group seeds: landing (the map units in view when you land; "comedown" is its old
    /// name), layout (CaveGen's default), rooms, or first (the first room placed).
    #[arg(long, default_value = "landing", value_parser = ["landing", "comedown", "layout", "rooms", "first"])]
    group: String,

    #[command(flatten)]
    range: Range,

    /// The game's screen shape, for the landing view: 16:9 or 4:3.
    #[arg(long, default_value = "16:9", value_parser = ["16:9", "4:3"])]
    screen: String,

    /// How many groups to draw.
    #[arg(long, default_value_t = 20)]
    max: usize,

    /// Hide objects in fewer than this percent of a group's seeds.
    #[arg(long, default_value_t = 2.0)]
    min_pct: f64,

    /// Kinds to leave off the maps, comma-separated: treasures, teki, gates, exits.
    #[arg(long, value_delimiter = ',')]
    hide: Vec<String>,

    /// Image size: 1 draws a map grid cell as 40 pixels.
    #[arg(long, default_value_t = 1.0)]
    scale: f32,

    /// The folder for the images.
    #[arg(short, long, default_value = "output")]
    out: PathBuf,
}

fn parse_seed(text: &str) -> Result<u32, String> {
    parse_seed_for(Generator::Gc, text).map(|s| s as u32)
}

/// A seed in hex: up to 8 digits on GC, 16 on Switch.
fn parse_seed_for(generator: Generator, text: &str) -> Result<u64, String> {
    let digits = generator.seed_digits();
    let bad = || format!("{text:?} isn't a seed: give up to {digits} hex digits");
    let t = text.trim();
    let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    if t.is_empty() || t.len() > digits {
        return Err(bad());
    }
    u64::from_str_radix(t, 16).map_err(|_| bad())
}


fn random_seed() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    seed_at("r", (nanos ^ (std::process::id() as u128) << 64) as u64, 0)
}

/// Where a slice of seeds starts, `offset` seeds into a range: counting and random seeds are
/// numbered; game-RNG seeds continue the RNG.
fn slice_start(mode: &str, start: u64, offset: u64) -> u64 {
    if mode == "g" {
        let mut rng = Rng::new(start as u32);
        rng.advance(offset);
        rng.seed() as u64
    } else {
        start + offset
    }
}

fn file_name(sublevel: &str) -> String {
    sublevel
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

fn write_png(out: &Path, name: &str, png: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let path = out.join(name);
    std::fs::write(&path, png).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let data = cli.data.clone().unwrap_or_else(default_data_dir);
    let cmd = cli.cmd.unwrap_or(Cmd::Generate(cli.generate));
    if matches!(cmd, Cmd::Options) || matches!(&cmd, Cmd::Generate(g) if g.list_options) {
        println!("Map options for --show and --hide (on by default: water, treasures, teki, plants, gates, gateHealth, exits):");
        for name in OPTION_NAMES {
            println!("  {name}");
        }
        return Ok(());
    }
    let mut painter = Painter::new(&data)?;
    painter.generator = cli.rng;
    if cli.rng != Generator::Gc && !matches!(cmd, Cmd::Generate(_) | Cmd::Judge { .. } | Cmd::Caves { .. } | Cmd::Caveinfo { .. }) {
        return Err(format!("--rng {} works only for drawing layouts and judge so far", cli.rng.name()));
    }
    match cmd {
        Cmd::Generate(args) => generate(&painter, &args),
        Cmd::Caves { game } => caves(&painter, game.as_deref()),
        Cmd::Caveinfo { sublevel, json } => caveinfo(&painter, &sublevel, json),
        Cmd::Judge { sublevel, judge, seeds } => {
            for s in &seeds {
                let seed = parse_seed_for(cli.rng, s)?;
                let r = painter.query(&["judge", &sublevel, &judge, &seed.to_string(), cli.rng.name()])?;
                println!("{seed:0w$X}  {}", r["score"], w = cli.rng.seed_digits());
            }
            Ok(())
        }
        Cmd::Search(args) => search_command(&painter, &args),
        Cmd::Stats { sublevel, query, range } => {
            let args = SearchArgs {
                sublevel,
                judge: None,
                query: Some(query),
                count: None,
                range,
                top: 0,
                images: None,
                look: LookArgs {
                    scale: 1.0,
                    view: None,
                    landing: false,
                    screen: "16:9".into(),
                    show: vec![],
                    hide: vec![],
                },
                json: false,
            };
            search_command(&painter, &args)
        }
        Cmd::Count { sublevel, what, range } => {
            let args = SearchArgs {
                sublevel,
                judge: None,
                query: None,
                count: Some(what),
                range,
                top: 0,
                images: None,
                look: LookArgs {
                    scale: 1.0,
                    view: None,
                    landing: false,
                    screen: "16:9".into(),
                    show: vec![],
                    hide: vec![],
                },
                json: false,
            };
            search_command(&painter, &args)
        }
        Cmd::Aggregate(args) => aggregate(&mut painter, &args),
        Cmd::Options => unreachable!(),
    }
}

fn generate(painter: &Painter, args: &GenerateArgs) -> Result<(), String> {
    let opts = args.look.options()?;
    let sublevel = args.sublevel.as_deref().unwrap_or_default();
    let g = painter.generator;
    let start = match &args.seed {
        Some(s) => parse_seed_for(g, s)?,
        None if g == Generator::Switch => mix64(random_seed() as u64 ^ (std::process::id() as u64) << 32),
        None => random_seed() as u64,
    };
    let began = Instant::now();
    let results: Vec<Result<PathBuf, String>> = (0..args.count)
        .into_par_iter()
        .map(|i| {
            let seed = seed_in(g, &args.seeds, start, i);
            let (layout, png) = painter.png(sublevel, seed, &opts)?;
            write_png(&args.out, &format!("{}_{}.png", file_name(&layout.sublevel), layout.seed), &png)
        })
        .collect();
    let mut written = 0;
    for r in results {
        let path = r?;
        if args.count <= 20 {
            println!("{}", path.display());
        }
        written += 1;
    }
    if args.count > 20 {
        println!("Wrote {written} images to {} in {:.1?}", args.out.display(), began.elapsed());
    }
    Ok(())
}

fn caves(painter: &Painter, game: Option<&str>) -> Result<(), String> {
    let list: Value = serde_json::from_str(&painter.web.caves_json()?).map_err(|e| e.to_string())?;
    let mut last_game = String::new();
    for c in list.as_array().into_iter().flatten() {
        let g = c["game"].as_str().unwrap_or_default();
        if game.is_some_and(|want| !want.eq_ignore_ascii_case(g)) {
            continue;
        }
        if g != last_game {
            println!("{}{g}:", if last_game.is_empty() { "" } else { "\n" });
            last_game = g.to_string();
        }
        let mode = if c["challenge"].as_bool() == Some(true) {
            "  (challenge mode)"
        } else {
            ""
        };
        println!(
            "  {:<8} {}{mode}",
            c["short"].as_str().unwrap_or_default(),
            c["name"].as_str().unwrap_or_default()
        );
    }
    if last_game.is_empty() {
        return Err(format!("no caves for {:?}", game.unwrap_or_default()));
    }
    println!("\nName a sublevel as <short>-<floor> (SCx-6); romhacks as <game>:<short>-<floor> (251:AT-2).");
    Ok(())
}

fn caveinfo(painter: &Painter, sublevel: &str, json: bool) -> Result<(), String> {
    let ci = painter.query(&["caveinfo", sublevel])?;
    if json {
        println!("{}", serde_json::to_string_pretty(&ci).map_err(|e| e.to_string())?);
        return Ok(());
    }
    let layout = painter.layout(sublevel, 0, false)?;
    let fl = &ci["floor"];
    let yes = |v: &Value| if v.as_bool() == Some(true) { "yes" } else { "no" };
    let pct = |v: &Value| format!("{}%", (v.as_f64().unwrap_or(0.0) * 1000.0).round() / 10.0);
    println!("{} ({})", layout.long_name, layout.sublevel);
    println!(
        "Max teki {}  Max treasures {}  Max gates {}  Rooms {}  Corridor chance {}  Alcove chance {}",
        fl["max_teki"],
        fl["max_treasures"],
        fl["max_gates"],
        fl["rooms"],
        pct(&fl["corridor_probability"]),
        pct(&fl["cap_probability"])
    );
    let mut flags = format!(
        "Geyser {}  Plugged hole {}  Final floor {}",
        yes(&fl["geyser"]),
        yes(&fl["plugged"]),
        yes(&fl["final"])
    );
    if fl["waterwraith"].as_f64().unwrap_or(0.0) > 0.0 {
        flags.push_str(&format!("  Waterwraith {} s", fl["waterwraith"]));
    }
    println!("{flags}");

    let group_name = |g: i64| match g {
        0 => "easy",
        1 => "hard",
        5 => "seam",
        6 => "plant",
        8 => "special",
        _ => "other",
    };
    let teki_line = |t: &Value, group: String| {
        let mut name = painter.name_and_id("teki", t["name"].as_str().unwrap_or_default());
        if let Some(c) = t["carrying"].as_str() {
            name.push_str(&format!(", carrying {}", painter.name("treasures", c)));
        }
        if t["falling"].as_bool() == Some(true) {
            name.push_str(", falls");
        }
        format!("  x{:<3} w{:<4} {:<8} {name}", t["min"].to_string(), t["weight"].to_string(), group)
    };
    let teki = ci["teki"].as_array().cloned().unwrap_or_default();
    if !teki.is_empty() {
        println!("\nTeki (min, weight, group):");
        for t in &teki {
            println!("{}", teki_line(t, group_name(t["group"].as_i64().unwrap_or(-1)).to_string()));
        }
    }
    let items = ci["items"].as_array().cloned().unwrap_or_default();
    if !items.is_empty() {
        println!("\nTreasures (min, weight, value, carry):");
        for t in &items {
            let carry = &t["carry"];
            println!(
                "  x{:<3} w{:<4} {:<5} {:<6} {}",
                t["min"].to_string(),
                t["weight"].to_string(),
                t["value"].to_string(),
                format!("{}/{}", carry[0], carry[1]),
                painter.name_and_id("treasures", t["name"].as_str().unwrap_or_default())
            );
        }
    }
    let gates = ci["gates"].as_array().cloned().unwrap_or_default();
    if !gates.is_empty() {
        println!("\nGates (HP, weight):");
        for g in &gates {
            println!("  {:<6} w{}", g["health"].as_f64().unwrap_or(0.0).round(), g["weight"]);
        }
    }
    let caps = ci["caps"].as_array().cloned().unwrap_or_default();
    if !caps.is_empty() {
        println!("\nAlcove teki (min, weight, group):");
        for t in &caps {
            println!("{}", teki_line(t, t["group"].to_string()));
        }
    }
    println!("\nMap units (size, type, doors):");
    for u in ci["units"].as_array().into_iter().flatten() {
        println!(
            "  {:<5} {:<8} {} doors  {}",
            format!("{}x{}", u["w"], u["h"]),
            u["type"].as_str().unwrap_or_default(),
            u["doors"],
            u["name"].as_str().unwrap_or_default()
        );
    }
    Ok(())
}

/// A search over many seeds, run in parallel slices.
struct SearchResult {
    tested: u64,
    matched: u64,
    /// (seed, score), best first; score 0 without a judge.
    best: Vec<(u32, f64)>,
    scores: Vec<f64>,
    counts: BTreeMap<u64, u64>,
}

fn search(painter: &Painter, args: &SearchArgs) -> Result<SearchResult, String> {
    let start = args.range.start()?;
    let mode = args.range.seeds.as_str();
    let total = args.range.num;
    let keep = args.top.max(1);
    const SLICE: u64 = 4096;
    let slices: Vec<u64> = (0..total.div_ceil(SLICE)).collect();
    let judge = args.judge.clone().unwrap_or_else(|| "-".into());
    let query = args.query.clone().unwrap_or_else(|| "-".into());
    let count = args.count.clone().unwrap_or_else(|| "-".into());
    let parts: Vec<Result<Value, String>> = slices
        .par_iter()
        .map(|&i| {
            let offset = i * SLICE;
            let n = SLICE.min(total - offset);
            let from = slice_start(mode, start, offset).to_string();
            painter.query(&[
                "search",
                &args.sublevel,
                &judge,
                &query,
                &from,
                &n.to_string(),
                mode,
                &keep.to_string(),
                &count,
            ])
        })
        .collect();
    let mut out = SearchResult {
        tested: 0,
        matched: 0,
        best: Vec::new(),
        scores: Vec::new(),
        counts: BTreeMap::new(),
    };
    for part in parts {
        let r = part?;
        out.tested += r["tested"].as_u64().unwrap_or(0);
        out.matched += r["matched"].as_u64().unwrap_or(0);
        for b in r["best"].as_array().into_iter().flatten() {
            out.best.push((b[0].as_u64().unwrap_or(0) as u32, b[1].as_f64().unwrap_or(0.0)));
        }
        out.scores
            .extend(r["scores"].as_array().into_iter().flatten().filter_map(Value::as_f64));
        for c in r["counts"].as_array().into_iter().flatten() {
            *out.counts.entry(c[0].as_u64().unwrap_or(0)).or_default() += c[1].as_u64().unwrap_or(0);
        }
    }
    if args.judge.is_some() {
        out.best.sort_by(|a, b| a.1.total_cmp(&b.1));
    }
    out.best.truncate(args.top);
    out.scores.sort_by(f64::total_cmp);
    Ok(out)
}

fn search_command(painter: &Painter, args: &SearchArgs) -> Result<(), String> {
    if args.judge.is_none() && args.query.is_none() && args.count.is_none() {
        return Err("give --judge, --query or --count".into());
    }
    let began = Instant::now();
    let r = search(painter, args)?;
    let secs = began.elapsed().as_secs_f64();
    let rank = |score: f64| {
        let at = r.scores.partition_point(|&s| s <= score);
        100.0 * at as f64 / r.scores.len().max(1) as f64
    };
    if args.json {
        let best: Vec<Value> = r
            .best
            .iter()
            .map(|(seed, score)| serde_json::json!({ "seed": format!("{seed:08X}"), "score": score, "rank_pct": rank(*score) }))
            .collect();
        let counts: Vec<Value> = r.counts.iter().map(|(k, n)| serde_json::json!([k, n])).collect();
        let out = serde_json::json!({ "tested": r.tested, "matched": r.matched, "best": best, "counts": counts });
        println!("{}", serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?);
    } else {
        println!("Tested {} seeds of {} in {secs:.1} s.", r.tested, args.sublevel);
        if args.query.is_some() {
            println!(
                "{} matched ({:.2}%){}.",
                r.matched,
                100.0 * r.matched as f64 / r.tested.max(1) as f64,
                if args.query.is_some() {
                    format!(" the query {:?}", args.query.as_deref().unwrap_or_default())
                } else {
                    String::new()
                }
            );
        }
        if !r.best.is_empty() {
            println!();
            for (i, (seed, score)) in r.best.iter().enumerate() {
                if args.judge.is_some() {
                    let p = rank(*score);
                    let p = if p < 1.0 { format!("{p:.2}") } else { format!("{p:.1}") };
                    println!("{:>3}. {seed:08X}  {score:<10} top {p}%", i + 1);
                } else {
                    println!("{:>3}. {seed:08X}", i + 1);
                }
            }
        }
        if let Some(what) = &args.count {
            let tested: u64 = r.counts.values().sum();
            let most = r.counts.values().copied().max().unwrap_or(1);
            println!("\nHow many {what}, over {tested} seeds:");
            for (k, n) in &r.counts {
                let bar = "#".repeat(((40 * n) / most.max(1)) as usize);
                println!("  {k:>3}  {n:>9} seeds  {:>6.2}%  {bar}", 100.0 * *n as f64 / tested.max(1) as f64);
            }
        }
    }
    if let Some(dir) = &args.images {
        let opts = args.look.options()?;
        for (seed, _) in &r.best {
            let (layout, png) = painter.png(&args.sublevel, *seed as u64, &opts)?;
            let path = write_png(dir, &format!("{}_{}.png", file_name(&layout.sublevel), layout.seed), &png)?;
            eprintln!("{}", path.display());
        }
    }
    Ok(())
}

fn aggregate(painter: &mut Painter, args: &AggArgs) -> Result<(), String> {
    let aspect = if args.screen == "4:3" { 4.0 / 3.0 } else { 16.0 / 9.0 };
    let shape = render::landing_shape(aspect);
    let periods = painter.image_periods(&args.sublevel)?;
    let game = painter.layout(&args.sublevel, 0, false)?.game.clone();
    painter.web.command(
        &[
            "agg_start",
            &args.sublevel,
            &args.group,
            &shape.near.to_string(),
            &shape.far.to_string(),
            &shape.near_half.to_string(),
            &shape.far_half.to_string(),
            &periods,
        ]
        .join("\t"),
    )?;
    let start = args.range.start()?;
    let mode = args.range.seeds.as_str();
    let total = args.range.num;
    let began = Instant::now();
    let mut done = 0;
    while done < total {
        let n = (total - done).min(20000);
        let from = slice_start(mode, start, done);
        painter.web.command(&format!("agg_run\t{from}\t{n}\t{mode}"))?;
        done += n;
        eprint!("\r{done} of {total} seeds");
    }
    eprintln!();
    let result: AggResult = serde_json::from_str(&painter.web.command(&format!("agg_result\t{}", args.max))?)
        .map_err(|e| format!("bad aggregator JSON: {e}"))?;
    let hidden = |kind: &str| {
        args.hide
            .iter()
            .any(|h| h.trim().trim_end_matches('s').eq_ignore_ascii_case(kind.trim_end_matches('s')))
    };
    let opts = AggOptions {
        treasures: !hidden("treasures"),
        teki: !hidden("teki"),
        gates: !hidden("gates"),
        exits: !hidden("exits"),
        min_share: args.min_pct / 100.0,
        scale: args.scale,
        game,
    };
    println!(
        "{} seeds of {} in {:.1} s, {} groups (by {}).",
        result.total,
        args.sublevel,
        began.elapsed().as_secs_f64(),
        result.groups_total,
        args.group
    );
    if result.dropped > 0 {
        println!("{} seeds didn't fit in the first 20,000 groups and aren't shown.", result.dropped);
    }
    let base = file_name(&args.sublevel);
    for (i, group) in result.groups.iter().enumerate() {
        let pixmap = render::render_agg_group(group, result.view, &opts, &painter.assets)?;
        let png = pixmap.encode_png().map_err(|e| e.to_string())?;
        let path = write_png(&args.out, &format!("{base}_{}_{:02}.png", args.group, i + 1), &png)?;
        let examples: Vec<String> = group.seeds.iter().take(4).map(|s| format!("{s:08X}")).collect();
        println!(
            "#{:<3} {:>5}% ({} seeds)  e.g. {}  {}",
            i + 1,
            pct(group.count, result.total),
            group.count,
            examples.join(" "),
            path.display()
        );
    }
    Ok(())
}
