//! Pikmin 2's random number generator and the helpers cave generation builds on it.
//!
//! Ported from Caveripper's `pikmin_math::rng` (MIT, https://github.com/mayabyte/caveripper).
//! The arithmetic is unchanged because every result must match the game bit for bit.

/// The game's linear congruential generator.
///
/// A layout's "seed" is this generator's state when the game starts generating the cave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    seed: u32,
}

const A: u32 = 0x41C6_4E6D;
const C: u32 = 0x3039;

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// The current internal state.
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// One step of the game's RNG, returning a value in `0..32768`.
    #[inline]
    pub fn rand_raw(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(A).wrapping_add(C);
        (self.seed >> 16) & 0x7FFF
    }

    /// Advances the state by `n` steps in O(log n) and returns the value the last step would
    /// have returned (or the current value's bits for `n == 0`).
    pub fn advance(&mut self, n: u32) -> u32 {
        // Compose x -> a*x + c with itself by repeated squaring.
        let (mut mul, mut add) = (1u32, 0u32);
        let (mut step_mul, mut step_add) = (A, C);
        let mut n = n;
        while n > 0 {
            if n & 1 == 1 {
                mul = mul.wrapping_mul(step_mul);
                add = add.wrapping_mul(step_mul).wrapping_add(step_add);
            }
            step_add = step_add.wrapping_mul(step_mul).wrapping_add(step_add);
            step_mul = step_mul.wrapping_mul(step_mul);
            n >>= 1;
        }
        self.seed = self.seed.wrapping_mul(mul).wrapping_add(add);
        (self.seed >> 16) & 0x7FFF
    }

    /// The game's `randInt`: a value in `0..max`.
    #[inline]
    pub fn rand_int(&mut self, max: u32) -> u32 {
        (self.rand_raw() as f32 * (max as f32 / 32768f32)) as u32
    }

    /// The game's `randFloat`: a value in `[0, 1)`.
    #[inline]
    pub fn rand_f32(&mut self) -> f32 {
        self.rand_raw() as f32 / 32768f32
    }

    /// Moves a random element to the back of the list, `n` times.
    pub fn rand_backs_n<T>(&mut self, list: &mut Vec<T>, n: usize) {
        for _ in 0..n {
            let index = self.rand_int(list.len() as u32) as usize;
            let elem = list.remove(index);
            list.push(elem);
        }
    }

    /// [`Rng::rand_backs_n`] once per element.
    pub fn rand_backs<T>(&mut self, list: &mut Vec<T>) {
        self.rand_backs_n(list, list.len())
    }

    /// Swaps each element with a random element.
    pub fn rand_swaps<T>(&mut self, list: &mut [T]) {
        for i in 0..list.len() {
            let swap_to = self.rand_int(list.len() as u32) as usize;
            list.swap(i, swap_to);
        }
    }

    /// Weighted random choice. Always makes one RNG call, even when the weights sum to 0.
    pub fn rand_index_weight(&mut self, weights: &[u32]) -> Option<usize> {
        let total: u32 = weights.iter().sum();
        let threshold = self.rand_int(total);
        let mut cumulative = 0;
        weights.iter().position(|w| {
            cumulative += w;
            cumulative > threshold
        })
    }

    /// Weighted random choice over signed weights, the way the game picks treasure spots.
    /// Makes no RNG call when the weights sum to 0, and returns None if no running total
    /// passes the random value (possible when some weights are negative).
    pub fn rand_index_weight_signed(&mut self, weights: &[i32]) -> Option<usize> {
        let total: i32 = weights.iter().sum();
        if total == 0 {
            return None;
        }
        let threshold = (self.rand_raw() as f32 * (total as f32 / 32768f32)) as i32;
        let mut cumulative = 0;
        weights.iter().position(|w| {
            cumulative += w;
            cumulative > threshold
        })
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use flate2::read::GzDecoder;

    use super::Rng;

    const TEST_SEED: u32 = 0x12345678;

    /// Reads one of the gzipped reference files made with the game's RNG.
    fn reference(name: &str) -> String {
        let path = format!("{}/../../reference/{name}.txt.gz", env!("CARGO_MANIFEST_DIR"));
        let mut text = String::new();
        GzDecoder::new(File::open(&path).unwrap()).read_to_string(&mut text).unwrap();
        text
    }

    fn lists(name: &str) -> Vec<Vec<u32>> {
        reference(name)
            .lines()
            .map(|line| line.split_terminator(',').map(|n| n.parse().unwrap()).collect())
            .collect()
    }

    #[test]
    fn raw_values_match_the_game() {
        let expected: Vec<u32> = reference("rng_200k").lines().map(|l| l.parse().unwrap()).collect();
        let mut rng = Rng::new(TEST_SEED);
        let actual: Vec<u32> = (0..expected.len()).map(|_| rng.rand_raw()).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn rand_int_matches_the_game() {
        let expected: Vec<u32> = reference("randInt_65k").lines().map(|l| l.parse().unwrap()).collect();
        let mut rng = Rng::new(TEST_SEED);
        let actual: Vec<u32> = (0..expected.len() as u32).map(|i| rng.rand_int(i)).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn rand_f32_matches_the_game() {
        let expected: Vec<f32> = reference("randFloat_100k").lines().map(|l| l.parse().unwrap()).collect();
        let mut rng = Rng::new(TEST_SEED);
        let actual: Vec<f32> = (0..expected.len()).map(|_| rng.rand_f32()).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn rand_backs_matches_the_game() {
        let expected = lists("randBacks_500");
        let mut rng = Rng::new(TEST_SEED);
        let actual: Vec<Vec<u32>> = (0..expected.len() as u32)
            .map(|i| {
                let mut list = (0..i).collect();
                rng.rand_backs(&mut list);
                list
            })
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn rand_swaps_matches_the_game() {
        let expected = lists("randSwaps_500");
        let mut rng = Rng::new(TEST_SEED);
        let actual: Vec<Vec<u32>> = (0..expected.len() as u32)
            .map(|i| {
                let mut list: Vec<u32> = (0..i).collect();
                rng.rand_swaps(&mut list);
                list
            })
            .collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn advance_matches_single_steps() {
        for n in [0u32, 1, 2, 15, 16, 17, 100, 12345, 1 << 20, u32::MAX] {
            let mut stepped = Rng::new(TEST_SEED);
            let mut last = 0;
            // Stepping 2^32-1 times is too slow, so check the big jump via its inverse below.
            if n <= 1 << 20 {
                for _ in 0..n {
                    last = stepped.rand_raw();
                }
                let mut jumped = Rng::new(TEST_SEED);
                let value = jumped.advance(n);
                assert_eq!(stepped, jumped, "n = {n}");
                if n > 0 {
                    assert_eq!(last, value, "n = {n}");
                }
            } else {
                // The LCG has period 2^32, so advancing 2^32-1 steps then one more returns to start.
                let mut rng = Rng::new(TEST_SEED);
                rng.advance(n);
                rng.rand_raw();
                assert_eq!(rng.seed(), TEST_SEED);
            }
        }
    }
}
