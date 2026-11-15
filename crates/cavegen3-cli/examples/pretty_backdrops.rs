//! Cuts the floor backdrops for the Pretty view (toy and grassy caves) out of CaveGen's
//! `files/gc/enemytex/special/{toy_bg,kusachi_bg}.png`.
//!
//! Those pictures are a floor texture with a frame around it (a wooden border for toy, the
//! skybox's walls for grass). Each crop is exactly a whole number of the floor texture's
//! repeats, so the backdrop tiles without seams; drawn at 40 pixels per map cell, as CaveGen
//! does (its crops shown at 2x, at 80 pixels per cell).
//!
//!   pretty_backdrops <special folder> <output folder>     write the cropped PNGs
//!   pretty_backdrops period <png> x0 y0 x1 y1             find a region's repeats (for tuning)
//! Then convert to WebP as `data/resources/pretty` holds them, e.g.
//!   ffmpeg -i toy_bg.png -c:v libwebp -quality 82 toy_bg.webp

/// (name, x, y, width, height): toy wood repeats every 92 px across (5 repeats) and about 221 px
/// down (2); grass every 339 px both ways.
const CROPS: [(&str, u32, u32, u32, u32); 2] = [("toy_bg", 80, 60, 460, 442), ("kusachi_bg", 164, 164, 339, 339)];

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.first().is_some_and(|m| m == "period") {
        return period(&a[1..]);
    }
    let [input, output] = &a[..] else {
        eprintln!("usage: pretty_backdrops <special folder> <output folder>");
        std::process::exit(2);
    };
    std::fs::create_dir_all(output).unwrap();
    for (name, x, y, w, h) in CROPS {
        let img = image::open(std::path::Path::new(input).join(format!("{name}.png"))).unwrap().to_rgba8();
        let mut crop = image::imageops::crop_imm(&img, x, y, w, h).to_image();
        // The toy floor has scattered see-through pixels: give each its neighbors' color.
        let src = crop.clone();
        for (px, py, p) in crop.enumerate_pixels_mut() {
            if p.0[3] == 255 {
                continue;
            }
            let (mut sum, mut n) = ([0u32; 3], 0);
            for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                let (qx, qy) = (px as i32 + dx, py as i32 + dy);
                if qx < 0 || qy < 0 || qx >= w as i32 || qy >= h as i32 {
                    continue;
                }
                let q = src.get_pixel(qx as u32, qy as u32).0;
                if q[3] == 255 {
                    (0..3).for_each(|i| sum[i] += q[i] as u32);
                    n += 1;
                }
            }
            assert!(n > 0, "{name}: the crop reaches past the floor at {px},{py}");
            *p = image::Rgba([(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8, 255]);
        }
        crop.save(std::path::Path::new(output).join(format!("{name}.png"))).unwrap();
        println!("{name}: {w}x{h}");
    }
}

/// Mean squared color difference between a region and itself shifted by each offset, best first.
fn period(a: &[String]) {
    let img = image::open(&a[0]).unwrap().to_rgba8();
    let n: Vec<u32> = a[1..5].iter().map(|s| s.parse().unwrap()).collect();
    let (x0, y0, x1, y1) = (n[0], n[1], n[2], n[3]);
    let diff = |dx: u32, dy: u32| -> f64 {
        let (mut s, mut c) = (0f64, 0f64);
        for y in (y0..y1.saturating_sub(dy)).step_by(2) {
            for x in (x0..x1.saturating_sub(dx)).step_by(2) {
                let (p, q) = (img.get_pixel(x, y).0, img.get_pixel(x + dx, y + dy).0);
                if p[3] < 250 || q[3] < 250 {
                    continue;
                }
                for i in 0..3 {
                    s += (p[i] as f64 - q[i] as f64).powi(2);
                }
                c += 1.0;
            }
        }
        if c < 1000.0 { f64::MAX } else { s / c }
    };
    for (label, across) in [("across", true), ("down", false)] {
        let span = if across { x1 - x0 } else { y1 - y0 };
        let mut v: Vec<(f64, u32)> = (8..span * 2 / 3).map(|d| (if across { diff(d, 0) } else { diff(0, d) }, d)).collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        let best: Vec<String> = v.iter().take(8).map(|(e, d)| format!("{d} px ({e:.0})")).collect();
        println!("{label}: {}", best.join(", "));
    }
}
