//! Studies of Pikmin 2 cave maps that close before all their rooms are placed and so can be short
//! of a treasure (or a boss): how likely that is, fast hunts for seeds that do it, and steering
//! generation by choosing what `rand()` returns. Built on CaveGen3's generator, through its
//! `internals` feature.

pub mod closure;
pub mod splitting;
pub mod steer;

/// splitmix64: a stream of well-mixed values from `state`.
pub(crate) fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
