//! Searching the seed space in parallel.
//!
//! Ported from Caveripper's `query::search` (MIT, https://github.com/mayabyte/caveripper).

use std::{
    hash::{BuildHasher, RandomState},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use rayon::scope;

use super::Query;
use crate::data::GameData;

/// Searches seeds on every thread for ones matching `query`, calling `on_found` for each.
///
/// - `deadline`: stop at this time; `None` searches until `num_to_find` seeds are found or the
///   whole space is covered.
/// - `num_to_find`: stop after this many matches; `None` for no limit.
/// - `on_tick`: called before checking each seed.
///
/// Each thread covers its own slice of the seed space (the top bit is unused by generation),
/// starting at a random point so repeated short searches find different seeds.
pub fn find_matching_layouts_parallel<T: Fn() + Send + Sync, F: Fn(u32) + Send + Sync>(
    query: &(impl Query + Send + Sync),
    data: &GameData,
    deadline: Option<Instant>,
    num_to_find: Option<usize>,
    on_tick: Option<T>,
    on_found: F,
) {
    let num_found = AtomicUsize::new(0);
    scope(|s| {
        s.spawn_broadcast(|_scope, ctx| {
            let slice = u32::MAX / 2 / (rayon::current_num_threads() as u32);
            let min_seed = slice * ctx.index() as u32;
            let max_seed = min_seed + slice - 1;
            let random = RandomState::new().hash_one(ctx.index()) as u32;
            let final_seed = min_seed + random % slice;
            let mut seed = final_seed;
            loop {
                if deadline.is_some_and(|d| Instant::now() > d) {
                    return;
                }
                if num_to_find.is_some_and(|n| num_found.load(Ordering::Relaxed) >= n) {
                    return;
                }
                if let Some(f) = on_tick.as_ref() {
                    f();
                }
                seed = if seed >= max_seed { min_seed } else { seed + 1 };
                if query.matches(seed, data) {
                    on_found(seed);
                    if num_to_find.is_some() {
                        num_found.fetch_add(1, Ordering::Relaxed);
                    }
                }
                if seed == final_seed {
                    return;
                }
            }
        });
    });
}
