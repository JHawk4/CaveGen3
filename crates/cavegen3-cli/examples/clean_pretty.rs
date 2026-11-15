//! Cleans CaveGen's "pretty" map unit pictures for CaveGen3's Pretty view.
//!
//! CaveGen made them from screenshots of each room taken against a purple background, with the
//! game's lighting. That leaves three things that look wrong when tiles sit side by side:
//!
//! 1. Purple bleed: where the floor wasn't drawn (the middle of the metal cross rooms) or blends
//!    into the background at an edge, the purple shows through. Those pixels get the tile's own
//!    floor color at the same brightness. Toy rooms really are purple in places (the play mat),
//!    so there only a thin band at the picture's edge is touched.
//! 2. A spotlight: each picture is brighter in the middle and darker toward its edges, and each
//!    tile's overall color differs a little. The brightness is evened out toward one level per
//!    tile set (conc, metal, snow, ...), and each tile's average color is nudged toward its tile
//!    set's, so neighbors match at their seams. Local color (puddles, rust) is kept.
//! 3. A dark band a little inside each picture's border, matched to the floor further in,
//!    and transparent edge pixels where the floor meets the edge, filled from the floor next to them.
//!
//! Usage: clean_pretty <CaveGen files/pretty folder> <output folder>
//! Then convert to WebP as `data/resources/pretty` holds them, e.g. with ffmpeg:
//!   ffmpeg -i in.png -c:v libwebp -quality 82 -pix_fmt yuva420p out.webp

use std::{collections::HashMap, path::PathBuf};

use image::{Rgba, RgbaImage};

/// How far a pixel is toward the screenshots' purple background: 0 none, 1 fully.
fn purpleness(p: &Rgba<u8>) -> f32 {
    let [r, g, b, _] = p.0;
    let excess = r.min(b) as f32 - g as f32;
    // Floors never have both red and blue above green, so even a little of that is bleed.
    ((excess - 6.0) / 24.0).clamp(0.0, 1.0)
}

fn lum(c: [f32; 3]) -> f32 {
    0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
}

fn is_floor(p: &Rgba<u8>) -> bool {
    p.0[3] > 200 && lum([p.0[0] as f32, p.0[1] as f32, p.0[2] as f32]) > 20.0
}

/// The tile set a unit belongs to: the last part of its name (way2_conc -> conc).
fn tileset(name: &str) -> &str {
    name.rsplit('_').next().unwrap_or(name)
}

fn is_toy(name: &str) -> bool {
    tileset(name) == "toy"
}

/// Distance in pixels to the picture's nearest edge.
fn edge_distance(x: u32, y: u32, w: u32, h: u32) -> u32 {
    x.min(y).min(w - 1 - x).min(h - 1 - y)
}

/// Whether a pixel counts as bleed for this tile.
fn bleed(name: &str, x: u32, y: u32, img: &RgbaImage) -> f32 {
    let (w, h) = img.dimensions();
    if is_toy(name) && edge_distance(x, y, w, h) > 4 {
        return 0.0;
    }
    purpleness(img.get_pixel(x, y))
}

/// Mean floor color of a tile, leaving out bleed.
fn mean_floor(name: &str, img: &RgbaImage) -> Option<[f32; 3]> {
    let mut sum = [0f64; 3];
    let mut n = 0f64;
    for (x, y, p) in img.enumerate_pixels() {
        if !is_floor(p) || bleed(name, x, y, img) > 0.0 {
            continue;
        }
        for c in 0..3 {
            sum[c] += p.0[c] as f64;
        }
        n += 1.0;
    }
    (n > 0.0).then(|| [(sum[0] / n) as f32, (sum[1] / n) as f32, (sum[2] / n) as f32])
}

fn clean(name: &str, img: &RgbaImage, target: [f32; 3]) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    let Some(mean) = mean_floor(name, img) else { return out };

    // 1. Purple bleed -> the tile's floor color at the pixel's own brightness.
    for y in 0..h {
        for x in 0..w {
            let p = *img.get_pixel(x, y);
            let s = bleed(name, x, y, img);
            if p.0[3] == 0 || s == 0.0 {
                continue;
            }
            let c = [p.0[0] as f32, p.0[1] as f32, p.0[2] as f32];
            let k = lum(c) / lum(mean).max(1.0);
            let mut q = p;
            for i in 0..3 {
                q.0[i] = (c[i] * (1.0 - s) + mean[i] * k * s).round().clamp(0.0, 255.0) as u8;
            }
            out.put_pixel(x, y, q);
        }
    }

    // 2. Even out the spotlight: divide each floor pixel's brightness by a heavily blurred
    // brightness of the floor around it, and move the tile's color toward its tile set's.
    let (wu, hu) = (w as usize, h as usize);
    let mut sat = vec![[0f64; 2]; (wu + 1) * (hu + 1)]; // summed-area table of (lum, count)
    for y in 0..hu {
        for x in 0..wu {
            let p = out.get_pixel(x as u32, y as u32);
            let v = if is_floor(p) { [lum([p.0[0] as f32, p.0[1] as f32, p.0[2] as f32]) as f64, 1.0] } else { [0.0, 0.0] };
            let i = (y + 1) * (wu + 1) + (x + 1);
            for k in 0..2 {
                sat[i][k] = v[k] + sat[i - 1][k] + sat[i - (wu + 1)][k] - sat[i - (wu + 1) - 1][k];
            }
        }
    }
    let r = ((w.max(h) as f32) * 0.2).round().max(4.0) as usize;
    let blurred = |x: usize, y: usize| -> Option<f32> {
        let (x0, y0, x1, y1) = (x.saturating_sub(r), y.saturating_sub(r), (x + r + 1).min(wu), (y + r + 1).min(hu));
        let at = |xx: usize, yy: usize| sat[yy * (wu + 1) + xx];
        let (a, b, c, d) = (at(x1, y1), at(x0, y1), at(x1, y0), at(x0, y0));
        let n = a[1] - b[1] - c[1] + d[1];
        (n > 0.0).then(|| ((a[0] - b[0] - c[0] + d[0]) / n) as f32)
    };
    let target_lum = lum(target);
    // Nearly colorless tile sets (snow) take their tile set's color fully, which removes the
    // purple cast some of their pictures have; colorful ones (rust and puddles) only 70% of
    // the way, since how much of each color a tile shows really differs.
    let colorless = target.iter().cloned().fold(f32::MIN, f32::max) - target.iter().cloned().fold(f32::MAX, f32::min) < 40.0;
    let tint: [f32; 3] = std::array::from_fn(|i| {
        let ratio = (target[i] / target_lum.max(1.0)) / (mean[i] / lum(mean).max(1.0)).max(0.01);
        if colorless { ratio.clamp(0.7, 1.4) } else { ratio.powf(0.7).clamp(0.8, 1.25) }
    });
    let src = out.clone();
    for y in 0..h {
        for x in 0..w {
            let p = *src.get_pixel(x, y);
            if !is_floor(&p) {
                continue;
            }
            let Some(local) = blurred(x as usize, y as usize) else { continue };
            let gain = (target_lum / local.max(1.0)).powf(0.85).clamp(0.6, 1.8);
            let mut q = p;
            let c = [p.0[0] as f32 * gain, p.0[1] as f32 * gain, p.0[2] as f32 * gain];
            // In a colorless tile set, a pixel takes the set's color at its own brightness,
            // keeping 30% of its own tint (the purple cast in snow rooms is in their pattern).
            let set_color = |i: usize| lum(c) * target[i] / target_lum.max(1.0);
            for i in 0..3 {
                let v = if colorless { set_color(i) * 0.7 + c[i] * tint[i] * 0.3 } else { c[i] * tint[i] };
                q.0[i] = v.round().clamp(0.0, 255.0) as u8;
            }
            out.put_pixel(x, y, q);
        }
    }

    // 3. The border: a dark band runs about 5-14 pixels in from each picture's edge. Each pixel
    // within 20 of the edge is scaled, per channel, to the color of the floor further in (18-31
    // pixels from the edge) at the same place along that edge (within 12 pixels either way),
    // so walls and doorways each keep their own level. Full strength up to 12 pixels in,
    // fading out by 20. The texture stays.
    const BAND: u32 = 20;
    const DEPTH: usize = 32;
    const SPREAD: i64 = 12;
    // Per side (top, bottom, left, right): [position along the edge][depth] -> (sum, count).
    let len = |side: usize| if side < 2 { w as usize } else { h as usize };
    let mut prof: Vec<Vec<[f64; 4]>> = (0..4).map(|side| vec![[0f64; 4]; len(side) * DEPTH]).collect();
    let side_of = |x: u32, y: u32| -> (usize, usize, u32) {
        let (dt, db, dl, dr) = (y, h - 1 - y, x, w - 1 - x);
        let d = dt.min(db).min(dl).min(dr);
        if d == dt {
            (0, x as usize, d)
        } else if d == db {
            (1, x as usize, d)
        } else if d == dl {
            (2, y as usize, d)
        } else {
            (3, y as usize, d)
        }
    };
    for (x, y, p) in out.enumerate_pixels() {
        if !is_floor(p) {
            continue;
        }
        let (side, t, d) = side_of(x, y);
        if (d as usize) < DEPTH {
            let e = &mut prof[side][t * DEPTH + d as usize];
            for i in 0..3 {
                e[i] += p.0[i] as f64;
            }
            e[3] += 1.0;
        }
    }
    // Mean color over positions t-SPREAD..=t+SPREAD and depths in `depths`.
    let window = |side: usize, t: usize, depths: std::ops::Range<usize>| -> Option<[f64; 3]> {
        let mut acc = [0f64; 4];
        let lo = (t as i64 - SPREAD).max(0) as usize;
        let hi = ((t as i64 + SPREAD) as usize).min(len(side) - 1);
        for tt in lo..=hi {
            for d in depths.clone() {
                let e = prof[side][tt * DEPTH + d];
                for i in 0..4 {
                    acc[i] += e[i];
                }
            }
        }
        (acc[3] >= 4.0).then(|| [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]])
    };
    let src = out.clone();
    for y in 0..h {
        for x in 0..w {
            let p = *src.get_pixel(x, y);
            let (side, t, d) = side_of(x, y);
            if d >= BAND || !is_floor(&p) {
                continue;
            }
            let (Some(reference), Some(here)) = (window(side, t, 18..DEPTH), window(side, t, d as usize..d as usize + 1)) else {
                continue;
            };
            let strength = if d < 12 { 1.0 } else { (BAND - d) as f64 / 8.0 };
            let mut q = p;
            for i in 0..3 {
                let full = (reference[i] / here[i].max(1.0)).clamp(0.7, 1.5);
                let gain = 1.0 + (full - 1.0) * strength;
                q.0[i] = (p.0[i] as f64 * gain).round().clamp(0.0, 255.0) as u8;
            }
            out.put_pixel(x, y, q);
        }
    }

    // 4. The outermost pixels are sometimes transparent where the floor runs to the edge, which
    // leaves a dark gap between tiles: fill them from the floor just inside.
    let src = out.clone();
    for y in 0..h {
        for x in 0..w {
            if edge_distance(x, y, w, h) >= 3 || src.get_pixel(x, y).0[3] == 255 {
                continue;
            }
            let (ix, iy) = (x.clamp(3, w.saturating_sub(4)), y.clamp(3, h.saturating_sub(4)));
            let inner = *src.get_pixel(ix, iy);
            if is_floor(&inner) {
                out.put_pixel(x, y, inner);
            }
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [input, output] = &args[..] else {
        eprintln!("usage: clean_pretty <CaveGen files/pretty folder> <output folder>");
        std::process::exit(2);
    };
    let mut tiles: Vec<(String, RgbaImage)> = Vec::new();
    for entry in std::fs::read_dir(input).expect("can't read the input folder") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "png") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            tiles.push((name, image::open(&path).expect("bad png").to_rgba8()));
        }
    }
    // Each tile set's target color: the average of its tiles' floor colors.
    let mut sets: HashMap<String, ([f32; 3], f32)> = HashMap::new();
    for (name, img) in &tiles {
        if let Some(m) = mean_floor(name, img) {
            let e = sets.entry(tileset(name).to_string()).or_insert(([0.0; 3], 0.0));
            for i in 0..3 {
                e.0[i] += m[i];
            }
            e.1 += 1.0;
        }
    }
    std::fs::create_dir_all(output).unwrap();
    for (name, img) in &tiles {
        let (sum, n) = sets.get(tileset(name)).copied().unwrap_or(([128.0; 3], 1.0));
        let target = [sum[0] / n, sum[1] / n, sum[2] / n];
        let out = clean(name, img, target);
        out.save(PathBuf::from(output).join(format!("{name}.png"))).unwrap();
    }
    let mut names: Vec<_> = sets.keys().cloned().collect();
    names.sort();
    println!("cleaned {} tiles in {} tile sets ({})", tiles.len(), names.len(), names.join(", "));
}
