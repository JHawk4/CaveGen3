//! Draws the layout a tape of chosen rand() values generates: `draw_tape SUBLEVEL gc|switch
//! TAPE_FILE OUT.png`. The tape file has one call per line; its second column is the value
//! (the format `steer` writes).
use cavegen3::Generator;
use cavegen3_cli::{Painter, default_data_dir, render};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let generator: Generator = a[2].parse().unwrap();
    let values: Vec<u32> = std::fs::read_to_string(&a[3])
        .unwrap()
        .lines()
        .filter_map(|l| l.split_whitespace().nth(1)?.parse().ok())
        .collect();
    let painter = Painter::new(&default_data_dir()).unwrap();
    let json = painter.web.generate_tape_json(&a[1], generator, &values, 0).unwrap();
    let layout: render::Layout = serde_json::from_str(&json).unwrap();
    let pixmap = render::render(&layout, &render::Options::default(), &painter.assets).unwrap();
    std::fs::write(&a[4], pixmap.encode_png().unwrap()).unwrap();
}
