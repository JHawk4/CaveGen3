//! Story sublevels where a map with only its start room (and no item alcoves) would be short of
//! treasure spots: `start_spots`. Prints the fewest spots a start room offers against the
//! treasures needed.
use cavegen3::{GameData, caveinfo::RoomType};
fn main() {
    let data = GameData::bundled().unwrap();
    for cave in data.caves() {
        if cave.game != "pikmin2" || cave.is_challenge_mode || cave.is_colossal_caverns() { continue; }
        for floor in 1.. {
            let Ok(sub) = data.sublevel(&format!("{}-{floor}", cave.short_name())) else { break };
            let Ok(ci) = data.caveinfo(&sub) else { break };
            if ci.max_treasures == 0 { continue; }
            let starts: Vec<(String, usize)> = ci.cave_units.iter()
                .filter(|u| u.room_type == RoomType::Room && u.spawnpoints.iter().any(|s| s.group == 7))
                .map(|u| (u.unit_folder_name.clone(), u.spawnpoints.iter().filter(|s| s.group == 2).count()))
                .collect();
            let min = starts.iter().map(|s| s.1).min().unwrap_or(0);
            let alcoves = ci.cave_units.iter().any(|u| u.unit_folder_name.contains("item"));
            if min < ci.max_treasures as usize {
                let mut names: Vec<String> = starts.iter().map(|s| format!("{} {}", s.0, s.1)).collect();
                names.sort(); names.dedup();
                println!("{:<7} rooms {} treasures {} corridor {:>3.0}%  item alcoves {}  start rooms: {}", sub.normalized_name(), ci.num_rooms, ci.max_treasures, 100.0 * ci.corridor_probability, alcoves, names.join(", "));
            }
        }
    }
}
