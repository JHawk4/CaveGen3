//! Treasure spots each unit of a sublevel offers: `treasure_spots SUBLEVEL...`. Rooms offer their
//! group 2 spawn points, item alcoves one spot each.
use cavegen3::{GameData, caveinfo::RoomType};
fn main() {
    let data = GameData::bundled().unwrap();
    for name in std::env::args().skip(1) {
        let ci = data.caveinfo(&data.sublevel(&name).unwrap()).unwrap();
        println!("{name}: {} rooms, {} treasures, corridor {:.0}%, cap {:.0}%", ci.num_rooms, ci.max_treasures, 100.0 * ci.corridor_probability, 100.0 * ci.cap_probability);
        let mut seen = std::collections::HashSet::new();
        for u in &ci.cave_units {
            if !seen.insert(u.unit_folder_name.clone()) { continue; }
            let spots = u.spawnpoints.iter().filter(|s| s.group == 2).count();
            let start = u.spawnpoints.iter().any(|s| s.group == 7);
            let kind = match u.room_type { RoomType::Room => "room", RoomType::DeadEnd => "cap", RoomType::Hallway => "hall" };
            if u.room_type != RoomType::Hallway {
                println!("  {kind:<4} {:<28} treasure spots {spots}{}{}", u.unit_folder_name, if start { " (start)" } else { "" }, if u.unit_folder_name.contains("item") { " ITEM ALCOVE" } else { "" });
            }
        }
    }
}
