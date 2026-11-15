use std::{path::PathBuf, process::ExitCode, time::Instant};

use cavegen3_cli::{
    Painter, default_data_dir,
    render::{OPTION_NAMES, Options},
};
use cavegen3_web::seed_at;
use clap::Parser;
use rayon::prelude::*;

/// Generates Pikmin 2 cave layouts and saves them as PNG images, drawn like the web page's map.
///
/// Examples:
///   cavegen3 SCx-6 1234ABCD
///   cavegen3 BK-4 --count 100 --seeds r --out bk4
///   cavegen3 SCx-6 1234ABCD --comedown --screen 4:3
///   cavegen3 FC-3 0 --show spawnpoints,doorScores,grid --hide water
#[derive(Parser)]
#[command(name = "cavegen3", version, verbatim_doc_comment)]
struct Args {
    /// The sublevel, like SCx-6, BK-1 or "Subterranean Complex 6". Romhacks take a game prefix,
    /// like 251:AT-2 or 216:CC-1; Colossal Caverns is "colossal".
    #[arg(required_unless_present = "list_options")]
    sublevel: Option<String>,

    /// The seed in hex (up to 8 digits, 0x optional). A random one if left out.
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

    /// Image size: 1 draws a map grid cell as 40 pixels.
    #[arg(long, default_value_t = 1.0)]
    scale: f32,

    /// Draw only what you see when you land, turned so the camera faces up.
    #[arg(long)]
    comedown: bool,

    /// The game's screen shape, for the comedown: 16:9 or 4:3.
    #[arg(long, default_value = "16:9", value_parser = ["16:9", "4:3"])]
    screen: String,

    /// Map options to turn on, comma-separated (see --list-options).
    #[arg(long, value_delimiter = ',')]
    show: Vec<String>,

    /// Map options to turn off, comma-separated.
    #[arg(long, value_delimiter = ',')]
    hide: Vec<String>,

    /// List the map options and exit.
    #[arg(long)]
    list_options: bool,

    /// The data folder (default: CAVEGEN3_DATA, else the repository's data folder).
    #[arg(long)]
    data: Option<PathBuf>,
}

fn parse_seed(text: &str) -> Result<u32, String> {
    let t = text.trim();
    let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
    if t.is_empty() || t.len() > 8 {
        return Err(format!("{text:?} isn't a seed: give up to 8 hex digits"));
    }
    u32::from_str_radix(t, 16).map_err(|_| format!("{text:?} isn't a seed: give up to 8 hex digits"))
}

fn random_seed() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    seed_at("r", (nanos ^ (std::process::id() as u128) << 64) as u64, 0)
}

fn run(args: Args) -> Result<(), String> {
    if args.list_options {
        println!("Map options for --show and --hide (on by default: water, treasures, teki, plants, gates, exits):");
        for name in OPTION_NAMES {
            println!("  {name}");
        }
        return Ok(());
    }
    let mut opts = Options {
        comedown: args.comedown,
        scale: args.scale,
        ..Options::default()
    };
    if args.screen == "4:3" {
        opts.aspect = 4.0 / 3.0;
    }
    if !(0.05..=20.0).contains(&args.scale) {
        return Err("--scale must be between 0.05 and 20".into());
    }
    for name in &args.show {
        opts.set(name, true)?;
    }
    for name in &args.hide {
        opts.set(name, false)?;
    }

    let sublevel = args.sublevel.as_deref().unwrap_or_default();
    let data = args.data.clone().unwrap_or_else(default_data_dir);
    let painter = Painter::new(&data)?;
    let start = match &args.seed {
        Some(s) => parse_seed(s)? as u64,
        None => random_seed() as u64,
    };
    std::fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;

    let began = Instant::now();
    let results: Vec<Result<PathBuf, String>> = (0..args.count)
        .into_par_iter()
        .map(|i| {
            let seed = seed_at(&args.seeds, start, i);
            let (layout, png) = painter.png(sublevel, seed, &opts)?;
            let name: String = layout
                .sublevel
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
                .collect();
            let path = args.out.join(format!("{name}_{}.png", layout.seed));
            std::fs::write(&path, png).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(path)
        })
        .collect();

    let mut written = 0;
    for r in &results {
        match r {
            Ok(path) if args.count <= 20 => {
                println!("{}", path.display());
                written += 1;
            }
            Ok(_) => written += 1,
            Err(e) => return Err(e.clone()),
        }
    }
    if args.count > 20 {
        println!("Wrote {written} images to {} in {:.1?}", args.out.display(), began.elapsed());
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
