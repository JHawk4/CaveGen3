//! Pikmin 2's random number generator and the helpers cave generation builds on it.
//!
//! Ported from Caveripper's `pikmin_math::rng` (MIT, https://github.com/mayabyte/caveripper).
//! The arithmetic is unchanged because every result must match the game bit for bit.
//! The Switch release uses a different `rand()` ([`Generator::Switch`]); everything built on it
//! (randInt, randFloat, the list shuffles) is the same code.

/// Which `rand()` the game was built with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Generator {
    /// GameCube, Wii and Wii U: Metrowerks MSL's `rand()`, a 32-bit LCG returning 15 bits.
    #[default]
    Gc,
    /// Switch (Pikmin 2 HD, 2023): musl libc's `rand()`, a 64-bit LCG returning 31 bits.
    ///
    /// Recovered from video (Challenge Mode score rolls and story-mode letter speeds), not from
    /// the Switch binary. The game's code is unchanged, so `randFloat` is still
    /// `(f32)rand() / RAND_MAX`, with the native `RAND_MAX` (2^31 - 1, which is 2^31 as an f32).
    Switch,
}

impl Generator {
    /// `"gc"` or `"switch"`.
    pub fn name(self) -> &'static str {
        match self {
            Generator::Gc => "gc",
            Generator::Switch => "switch",
        }
    }

    /// How many hex digits a seed has: 8 on GC, 16 on Switch.
    pub fn seed_digits(self) -> usize {
        match self {
            Generator::Gc => 8,
            Generator::Switch => 16,
        }
    }

    /// `RAND_MAX` as the game's float arithmetic sees it.
    #[inline]
    fn rand_max(self) -> f32 {
        match self {
            Generator::Gc => 32768f32,
            Generator::Switch => 2147483648f32,
        }
    }
}

impl std::str::FromStr for Generator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gc" | "gamecube" | "wii" | "wiiu" | "wii u" => Ok(Generator::Gc),
            "switch" | "nx" => Ok(Generator::Switch),
            other => Err(format!("unknown generator {other:?}: use gc or switch")),
        }
    }
}

impl Tape {
    #[cold]
    fn next(&mut self, generator: Generator) -> u32 {
        if self.pos == self.values.len() {
            // splitmix64
            self.fresh = self.fresh.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = self.fresh;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^= z >> 31;
            let v = match generator {
                Generator::Gc => (z >> 49) as u32,
                Generator::Switch => (z >> 33) as u32,
            };
            self.values.push(v);
        }
        self.pos += 1;
        self.values[self.pos - 1]
    }
}

/// The game's linear congruential generator.
///
/// A layout's "seed" is this generator's state when the game starts generating the cave: 32 bits
/// on GC, 64 on Switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
    generator: Generator,
    /// When set, `rand()` plays these values instead of stepping the LCG.
    tape: Option<Box<Tape>>,
}

/// A list of `rand()` values to play back, for studying what a generator would have to return:
/// past its end, fresh values come from a private random stream and are added to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tape {
    pub values: Vec<u32>,
    pub pos: usize,
    fresh: u64,
}

const A: u32 = 0x41C6_4E6D;
const C: u32 = 0x3039;
const A64: u64 = 6_364_136_223_846_793_005;
const C64: u64 = 1;

impl Rng {
    /// The GC generator at state `seed`.
    pub fn new(seed: u32) -> Self {
        Self { state: seed as u64, generator: Generator::Gc, tape: None }
    }

    /// Plays `values` as `rand()`'s results with `generator`'s arithmetic, then fresh random
    /// values drawn from a stream seeded by `fresh` (recorded onto the tape).
    pub fn tape(generator: Generator, values: Vec<u32>, fresh: u64) -> Self {
        Self { state: 0, generator, tape: Some(Box::new(Tape { values, pos: 0, fresh })) }
    }

    /// The tape, if this plays one.
    pub fn tape_ref(&self) -> Option<&Tape> {
        self.tape.as_deref()
    }

    /// Replaces where fresh tape values come from (so clones diverge).
    pub fn reseed_fresh(&mut self, fresh: u64) {
        if let Some(t) = self.tape.as_mut() {
            t.fresh = fresh;
        }
    }

    /// The (multiplier, increment) that steps `generator`'s state `n` times at once: state
    /// becomes `state * mul + add` (wrapping, cut to 32 bits on GC). For [`Rng::jump`].
    pub fn jump_coefficients(generator: Generator, n: u64) -> (u64, u64) {
        let (mut a, mut c) = match generator {
            Generator::Gc => (A as u64, C as u64),
            Generator::Switch => (A64, C64),
        };
        let (mut mul, mut add) = (1u64, 0u64);
        let mut n = n;
        while n > 0 {
            if n & 1 == 1 {
                mul = mul.wrapping_mul(a);
                add = add.wrapping_mul(a).wrapping_add(c);
            }
            c = c.wrapping_mul(a).wrapping_add(c);
            a = a.wrapping_mul(a);
            n >>= 1;
        }
        (mul, add)
    }

    /// Steps the state as many times as `jump_coefficients` was given, in one go (no tape).
    #[inline]
    pub fn jump(&mut self, (mul, add): (u64, u64)) {
        self.state = self.state.wrapping_mul(mul).wrapping_add(add);
        if self.generator == Generator::Gc {
            self.state &= 0xFFFF_FFFF;
        }
    }

    /// `generator` at state `seed` (only the low 32 bits count on GC).
    pub fn with(generator: Generator, seed: u64) -> Self {
        let state = match generator {
            Generator::Gc => seed as u32 as u64,
            Generator::Switch => seed,
        };
        Self { state, generator, tape: None }
    }

    pub fn generator(&self) -> Generator {
        self.generator
    }

    /// The current internal state, cut to 32 bits (all of it on GC).
    pub fn seed(&self) -> u32 {
        self.state as u32
    }

    /// The current internal state (64 bits on Switch).
    pub fn state(&self) -> u64 {
        self.state
    }

    /// One step of the game's RNG, returning `rand()`: a value in `0..32768` on GC, in `0..2^31`
    /// on Switch.
    #[inline]
    pub fn rand_raw(&mut self) -> u32 {
        if let Some(t) = self.tape.as_mut() {
            return t.next(self.generator);
        }
        match self.generator {
            Generator::Gc => {
                let s = (self.state as u32).wrapping_mul(A).wrapping_add(C);
                self.state = s as u64;
                (s >> 16) & 0x7FFF
            }
            Generator::Switch => {
                self.state = self.state.wrapping_mul(A64).wrapping_add(C64);
                (self.state >> 33) as u32
            }
        }
    }

    /// The value `rand()` returned when the state became what it is now.
    fn last_value(&self) -> u32 {
        match self.generator {
            Generator::Gc => ((self.state >> 16) & 0x7FFF) as u32,
            Generator::Switch => (self.state >> 33) as u32,
        }
    }

    /// Advances the state by `n` steps in O(log n) and returns the value the last step would
    /// have returned (or the current value's bits for `n == 0`).
    pub fn advance(&mut self, n: u64) -> u32 {
        // Compose x -> a*x + c with itself by repeated squaring.
        let (a, c, mask) = match self.generator {
            Generator::Gc => (A as u64, C as u64, u32::MAX as u64),
            Generator::Switch => (A64, C64, u64::MAX),
        };
        let (mut mul, mut add) = (1u64, 0u64);
        let (mut step_mul, mut step_add) = (a, c);
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
        // Arithmetic mod 2^64 reduces correctly mod 2^32 for GC.
        self.state = self.state.wrapping_mul(mul).wrapping_add(add) & mask;
        self.last_value()
    }

    /// The game's `randInt`: a value in `0..max`.
    ///
    /// On Switch, `(f32)rand()` rounds to 24 bits, so the top 64 values of `rand()` become
    /// 2^31 and `randInt` returns `max` itself (about once in 2^25 calls). The game then reads
    /// past the end of its list; this returns `max - 1` instead, the nearest valid index.
    #[inline]
    pub fn rand_int(&mut self, max: u32) -> u32 {
        let g = self.generator;
        let v = (self.rand_raw() as f32 * (max as f32 / g.rand_max())) as u32;
        if g == Generator::Switch && v >= max { max.saturating_sub(1) } else { v }
    }

    /// The game's `randFloat`: a value in `[0, 1)` (on Switch, rarely exactly 1).
    #[inline]
    pub fn rand_f32(&mut self) -> f32 {
        let g = self.generator;
        self.rand_raw() as f32 / g.rand_max()
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
        let rand_max = self.generator.rand_max();
        let threshold = (self.rand_raw() as f32 * (total as f32 / rand_max)) as i32;
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

    use super::{Generator, Rng};

    #[test]
    fn jump_matches_stepping() {
        for generator in [Generator::Gc, Generator::Switch] {
            for n in [0u64, 1, 2, 47, 1000, 123_457] {
                let mut stepped = Rng::with(generator, 0xDEAD_BEEF_1234_5678);
                for _ in 0..n {
                    stepped.rand_raw();
                }
                let mut jumped = Rng::with(generator, 0xDEAD_BEEF_1234_5678);
                jumped.jump(Rng::jump_coefficients(generator, n));
                assert_eq!(stepped.state(), jumped.state(), "{generator:?} {n}");
            }
        }
    }

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
                let value = jumped.advance(n as u64);
                assert_eq!(stepped, jumped, "n = {n}");
                if n > 0 {
                    assert_eq!(last, value, "n = {n}");
                }
            } else {
                // The LCG has period 2^32, so advancing 2^32-1 steps then one more returns to start.
                let mut rng = Rng::new(TEST_SEED);
                rng.advance(n as u64);
                rng.rand_raw();
                assert_eq!(rng.seed(), TEST_SEED);
            }
        }
    }

    // The Switch generator: musl's rand(), state = 6364136223846793005 * state + 1 (mod 2^64),
    // rand() = state >> 33.

    #[test]
    fn switch_raw_values_follow_musl() {
        let seed = 0x3063_06F6_6F04_43C4u64;
        let mut rng = Rng::with(Generator::Switch, seed);
        let got: Vec<u32> = (0..5).map(|_| rng.rand_raw()).collect();
        assert_eq!(got, [516786873, 1534398101, 1928869378, 239884336, 994856078]);
        assert_eq!(rng.state(), 0x7698_991D_C9C5_A8D1);
        let mut s = seed;
        let mut rng = Rng::with(Generator::Switch, seed);
        for _ in 0..10_000 {
            s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            assert_eq!(rng.rand_raw(), (s >> 33) as u32);
        }
    }

    #[test]
    fn switch_advance_matches_single_steps() {
        for n in [0u64, 1, 2, 63, 64, 65, 12345, 1 << 20] {
            let mut stepped = Rng::with(Generator::Switch, 0xFEDC_BA98_7654_3210);
            let mut last = 0;
            for _ in 0..n {
                last = stepped.rand_raw();
            }
            let mut jumped = Rng::with(Generator::Switch, 0xFEDC_BA98_7654_3210);
            let value = jumped.advance(n);
            assert_eq!(stepped, jumped, "n = {n}");
            if n > 0 {
                assert_eq!(last, value, "n = {n}");
            }
        }
    }

    /// randFloat and randInt with a 31-bit rand(): (f32)rand() / 2^31, in f32.
    #[test]
    fn switch_floats_round_like_the_game() {
        // A state whose next rand() is r: anything with r in bits 33..64 after one step.
        let rng_giving = |r: u32| {
            let target = (r as u64) << 33 | 0x1_2345_6789;
            let a_inv = {
                let a = 6_364_136_223_846_793_005u64;
                let mut x = a; // Newton's iteration for the inverse mod 2^64
                for _ in 0..6 {
                    x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
                }
                x
            };
            Rng::with(Generator::Switch, target.wrapping_sub(1).wrapping_mul(a_inv))
        };
        let mut r = rng_giving(0x4000_0000);
        assert_eq!(r.clone().rand_raw(), 0x4000_0000);
        assert_eq!(r.rand_f32(), 0.5);
        assert_eq!(rng_giving(0x4000_0000).rand_int(9), 4);
        // 0x7FFF_FFC0 and up round to 2^31 as an f32: randFloat is 1 and randInt would be max.
        assert_eq!(rng_giving(0x7FFF_FFC0).rand_f32(), 1.0);
        assert_eq!(rng_giving(0x7FFF_FFC0).rand_int(9), 8);
        assert!(rng_giving(0x7FFF_FFBF).rand_f32() < 1.0);
        // Challenge Mode digits: randInt(9) + 1 is floor(9 * (state >> 49) / 32768) + 1 to within
        // f32 rounding (the low 16 bits of rand() can tip it); check it on a stretch of the stream.
        let mut a = Rng::with(Generator::Switch, 42);
        let mut b = a.clone();
        let mut agree = 0;
        for _ in 0..100_000 {
            let d = a.rand_int(9);
            b.rand_raw();
            agree += (d as u64 == 9 * (b.state() >> 49) / 32768) as u32;
        }
        assert!(agree > 99_950, "{agree}"); // they differ about once in 10^4 draws
    }
}
