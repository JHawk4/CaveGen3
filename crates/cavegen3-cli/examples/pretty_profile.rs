//! Diagnostics for CaveGen's pretty tiles (for tuning clean_pretty).
//!   pretty_profile profile <png>...   color by distance from the edge
//!   pretty_profile purple <png>...    share of purple pixels, and a sample of them

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, files) = args.split_first().expect("usage: pretty_profile profile|purple <png>...");
    for path in files {
        let img = image::open(path).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        if mode == "purple" {
            let mut n = 0;
            let mut opaque = 0;
            let mut sample = None;
            for (x, y, p) in img.enumerate_pixels() {
                let [r, g, b, a] = p.0;
                if a < 200 {
                    continue;
                }
                opaque += 1;
                if r as i32 > g as i32 + 35 && b as i32 > g as i32 + 35 {
                    n += 1;
                    sample.get_or_insert((x, y, r, g, b));
                }
            }
            if n > 0 {
                let name = std::path::Path::new(path).file_stem().unwrap().to_string_lossy();
                println!("{name:<32} {:>6.2}%  e.g. {:?}", 100.0 * n as f64 / opaque.max(1) as f64, sample.unwrap());
            }
            continue;
        }
        let mut sums = vec![[0f64; 4]; 41];
        for (x, y, p) in img.enumerate_pixels() {
            let [r, g, b, a] = p.0;
            let lum = (r as u32 + g as u32 + b as u32) / 3;
            if a < 200 || lum < 25 {
                continue;
            }
            let d = x.min(y).min(w - 1 - x).min(h - 1 - y).min(40) as usize;
            let s = &mut sums[d];
            s[0] += r as f64;
            s[1] += g as f64;
            s[2] += b as f64;
            s[3] += 1.0;
        }
        println!("{path} {w}x{h}");
        for (d, s) in sums.iter().enumerate() {
            if s[3] > 0.0 && (d < 16 || d % 8 == 0) {
                println!("  d{d:>2}: {:>5.1} {:>5.1} {:>5.1}  n{}", s[0] / s[3], s[1] / s[3], s[2] / s[3], s[3]);
            }
        }
    }
}
