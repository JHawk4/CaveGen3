//! What a teki is, by its internal name rather than by the spawn group it was placed in.
//!
//! Caveinfo puts plants in teki groups and teki in the plant group (6) now and then: Clovers in
//! easy spots, Bomb-rocks and Fiery Blowhogs in plant spots, and so on. Where the object itself
//! matters (the map's teki and plant toggles, the comedown view) it is classified here.
//!
//! - **Plants** are the 17 teki whose managers are `Plants::Mgr` subclasses in the decomp
//!   (`include/Game/plantsMgr.h`). The decomp's namespaces `DiodeRed`, `DiodeGreen` and
//!   `Margaret` are spelled `DaiodoRed`, `DaiodoGreen` and `Magaret` in caveinfo.
//! - **Hazards** are the fire geyser, electric wire and gas pipe (Hiba, ElecHiba, GasHiba).
//! - Everything else is a regular teki, Candypop Buds (`*Pom`), Pellet Posies (`Pelplant`) and
//!   Skitter Leaves (`Sokkuri`) included: they are enemies in the game, with their own managers.
//!
//! The romhacks in `data/assets` add no new plant or hazard names. `web/app.js` keeps the same
//! lists (`PLANT_TEKI`, `HAZARD_TEKI`); a test checks they match.

/// Lowercase internal names of the plants.
pub const PLANTS: &[&str] = &[
    "chiyogami",
    "clover",
    "daiodogreen",
    "daiodored",
    "hikarikinoko",
    "kareooinu_l",
    "kareooinu_s",
    "magaret",
    "nekojarashi",
    "ooinu_l",
    "ooinu_s",
    "tanpopo",
    "tukushi",
    "wakame_l",
    "wakame_s",
    "watage",
    "zenmai",
];

/// Lowercase internal names of the hazards.
pub const HAZARDS: &[&str] = &["hiba", "elechiba", "gashiba"];

fn listed(list: &[&str], name: &str) -> bool {
    list.iter().any(|n| n.eq_ignore_ascii_case(name))
}

/// Whether a teki (internal name, any case) is a plant.
pub fn is_plant(name: &str) -> bool {
    listed(PLANTS, name)
}

/// Whether a teki (internal name, any case) is a fire, electric or gas hazard.
pub fn is_hazard(name: &str) -> bool {
    listed(HAZARDS, name)
}

/// Whether the game draws a teki while the pod comes down: plants and hazards, whose managers
/// derive from `EnemyMgrBaseAlwaysMovieActor`. Every other teki is hidden in that cutscene.
pub fn comedown_visible(name: &str) -> bool {
    is_plant(name) || is_hazard(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plants() {
        for n in ["Clover", "DaiodoRed", "DaiodoGreen", "Magaret", "HikariKinoko", "Ooinu_s", "KareOoinu_l", "Wakame_l", "Watage", "Chiyogami"] {
            assert!(is_plant(n), "{n}");
            assert!(comedown_visible(n), "{n}");
        }
        // Candypop Buds, Pellet Posies and Skitter Leaves are teki; hazards aren't plants.
        for n in ["BluePom", "RandPom", "Pelplant", "Sokkuri", "Hiba", "GasHiba", "Chappy", "Bomb", "Egg", "DiodeRed", "Margaret"] {
            assert!(!is_plant(n), "{n}");
        }
    }

    #[test]
    fn hazards() {
        for n in ["Hiba", "ElecHiba", "gashiba"] {
            assert!(is_hazard(n) && comedown_visible(n), "{n}");
        }
        for n in ["Bomb", "Clover", "Elecbug", "FireChappy"] {
            assert!(!is_hazard(n), "{n}");
        }
        assert!(!comedown_visible("Chappy") && !comedown_visible("BluePom") && !comedown_visible("Egg"));
    }

    /// web/app.js has the same lists.
    #[test]
    fn web_lists_match() {
        let js = include_str!("../../../web/app.js");
        for (var, list) in [("PLANT_TEKI", PLANTS), ("HAZARD_TEKI", HAZARDS)] {
            let start = js.find(&format!("const {var} = new Set([")).unwrap_or_else(|| panic!("{var} in app.js"));
            let body = &js[start..];
            let body = &body[body.find('[').unwrap() + 1..body.find(']').unwrap()];
            let mut got: Vec<&str> = body.split(',').map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty()).collect();
            let mut want = list.to_vec();
            got.sort();
            want.sort();
            assert_eq!(got, want, "{var}");
        }
    }
}
