//! CaveGen3: fast Pikmin 2 cave generation, seed search and layout judging.
//!
//! Parts of this crate are ported from Caveripper (https://github.com/mayabyte/caveripper,
//! MIT license); each such module says so at the top.

pub mod caveinfo;
pub mod data;
pub mod error;
pub mod generate;
pub mod jhawk_judge;
pub mod layout;
pub mod math;
mod parse;
pub mod point;
pub mod query;
pub mod rng;
pub mod waypoint;

pub use data::{CaveConfig, GameData, Sublevel, Variant};
pub use error::{Error, Result};
pub use layout::{Layout, Placement, SpawnObject};
pub use rng::Generator;
