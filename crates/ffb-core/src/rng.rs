use std::collections::VecDeque;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use crate::types::BlockResult;

// ── Roll spec (for deterministic injection) ───────────────────────────────────

/// A single deterministic dice value to inject via `TestRng`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollSpec {
    pub sides: u8,
    pub value: u8,
}

// ── GameRng ───────────────────────────────────────────────────────────────────

/// Pluggable RNG used throughout the game engine.
/// `Live` mode uses a seeded SmallRng (fast, not cryptographic — suitable for RL).
/// `Xoshiro` mode uses Xoshiro256** directly — identical roll sequence to the Java
///   Xoshiro256StarStar port, required for cross-engine parity testing.
/// `Test` mode replays a deterministic sequence; panics if the queue is exhausted.
pub enum GameRng {
    Live(SmallRng),
    Xoshiro(Xoshiro256StarStar),
    Test(VecDeque<u8>),
}

impl GameRng {
    pub fn new_live(seed: u64) -> Self {
        Self::Live(SmallRng::seed_from_u64(seed))
    }

    pub fn new_test(rolls: impl IntoIterator<Item = u8>) -> Self {
        Self::Test(rolls.into_iter().collect())
    }

    // ── Core roll ─────────────────────────────────────────────────────────

    /// Roll a fair die with `sides` faces, returning a value in `1..=sides`.
    pub fn roll(&mut self, sides: u8) -> u8 {
        assert!(sides >= 2, "die must have at least 2 sides");
        let result = match self {
            Self::Live(rng) => rng.gen_range(1..=sides),
            Self::Xoshiro(xo) => xo.roll(sides),
            Self::Test(q) => {
                let v = q.pop_front().expect("TestRng: dice queue exhausted");
                assert!(v >= 1 && v <= sides, "TestRng: value {v} out of range for d{sides}");
                v
            }
        };
        result
    }

    pub fn roll_d6(&mut self) -> u8 {
        self.roll(6)
    }

    pub fn roll_2d6(&mut self) -> u8 {
        self.roll_d6() + self.roll_d6()
    }

    pub fn roll_d8(&mut self) -> u8 {
        self.roll(8)
    }

    pub fn roll_2d8(&mut self) -> u8 {
        self.roll_d8() + self.roll_d8()
    }

    pub fn roll_d16(&mut self) -> u8 {
        self.roll(16)
    }

    pub fn roll_bool(&mut self, prob_num: u8, prob_den: u8) -> bool {
        self.roll(prob_den) <= prob_num
    }

    // ── Block dice ────────────────────────────────────────────────────────

    /// Roll `count` block dice (absolute value).
    /// Returns a `Vec<BlockResult>` sorted so that the relevant party can pick.
    ///
    /// Positive `count` → attacker rolls and picks (normal block).
    /// Negative `count` → defender rolls and picks (defender is stronger).
    pub fn roll_block_dice(&mut self, count: i8) -> Vec<BlockResult> {
        let n = count.unsigned_abs() as usize;
        assert!(n >= 1);
        (0..n).map(|_| BlockResult::from_d6(self.roll_d6())).collect()
    }

    // ── Direction ─────────────────────────────────────────────────────────

    /// Roll scatter direction: 1–8 (matches Blood Bowl scatter template).
    pub fn roll_scatter_direction(&mut self) -> u8 {
        self.roll(8)
    }

    /// Roll scatter distance: 1–6.
    pub fn roll_scatter_distance(&mut self) -> u8 {
        self.roll_d6()
    }

    // ── Xoshiro256** seeded constructor ──────────────────────────────────

    /// Build a `GameRng` backed by a Xoshiro256** stream.
    /// Produces the same roll sequence as the Java Xoshiro256StarStar port given
    /// the same seed — required for cross-engine parity testing.
    pub fn new_from_xoshiro(seed: u64) -> Self {
        Self::Xoshiro(Xoshiro256StarStar::new(seed))
    }

    // ── Seeded clone for parallelism ──────────────────────────────────────

    /// Derive a child RNG from a new seed. Used for root-parallel MCTS.
    pub fn child(&mut self, extra_seed: u64) -> Self {
        match self {
            Self::Live(rng) => {
                let base: u64 = rng.gen();
                Self::Live(SmallRng::seed_from_u64(base ^ extra_seed))
            }
            Self::Xoshiro(xo) => {
                let base = xo.next_u64();
                Self::Live(SmallRng::seed_from_u64(base ^ extra_seed))
            }
            Self::Test(_) => {
                // In test mode, children get an empty queue (they should not roll)
                Self::Test(VecDeque::new())
            }
        }
    }
}

// ── xoshiro256** — portable CSPRNG for cross-engine parity tests ──────────────

/// A simple xoshiro256** implementation identical to what will be ported to Java
/// for cross-engine deterministic parity testing.
pub struct Xoshiro256StarStar {
    state: [u64; 4],
}

impl Xoshiro256StarStar {
    pub fn new(seed: u64) -> Self {
        // Seed via SplitMix64 to avoid poor initial states
        let mut s = seed;
        let mut state = [0u64; 4];
        for word in &mut state {
            s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            *word = z ^ (z >> 31);
        }
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    /// Roll a fair die with `sides` faces using rejection sampling.
    pub fn roll(&mut self, sides: u8) -> u8 {
        let sides64 = sides as u64;
        let threshold = u64::MAX - (u64::MAX % sides64);
        loop {
            let v = self.next_u64();
            if v < threshold {
                return (v % sides64 + 1) as u8;
            }
        }
    }

    pub fn roll_d6(&mut self) -> u8 {
        self.roll(6)
    }

    /// Generate a sequence of N rolls on a d{sides} die.
    pub fn generate_sequence(&mut self, count: usize, sides: u8) -> Vec<u8> {
        (0..count).map(|_| self.roll(sides)).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TestRng ───────────────────────────────────────────────────────────

    #[test]
    fn test_rng_replays_sequence() {
        let mut rng = GameRng::new_test([3, 1, 6, 2, 5]);
        assert_eq!(rng.roll_d6(), 3);
        assert_eq!(rng.roll_d6(), 1);
        assert_eq!(rng.roll_d6(), 6);
        assert_eq!(rng.roll_d6(), 2);
        assert_eq!(rng.roll_d6(), 5);
    }

    #[test]
    #[should_panic(expected = "dice queue exhausted")]
    fn test_rng_panics_when_empty() {
        let mut rng = GameRng::new_test([1]);
        rng.roll_d6();
        rng.roll_d6(); // should panic
    }

    #[test]
    fn block_dice_returns_correct_count() {
        let mut rng = GameRng::new_test([1, 2, 3]);
        let dice = rng.roll_block_dice(3);
        assert_eq!(dice.len(), 3);
    }

    // ── LiveRng uniformity (chi-squared test) ─────────────────────────────

    #[test]
    fn d6_uniformity_chi_squared() {
        let mut rng = GameRng::new_live(12345);
        let n = 60_000usize;
        let mut counts = [0usize; 6];
        for _ in 0..n {
            counts[(rng.roll_d6() - 1) as usize] += 1;
        }
        let expected = n as f64 / 6.0;
        let chi_sq: f64 = counts.iter()
            .map(|&c| (c as f64 - expected).powi(2) / expected)
            .sum();
        // 5 degrees of freedom, 5% significance: critical value = 11.07
        assert!(chi_sq < 11.07, "chi-squared = {chi_sq:.3} exceeds 11.07 (d6 not uniform)");
    }

    #[test]
    fn d6_range() {
        let mut rng = GameRng::new_live(99);
        for _ in 0..10_000 {
            let v = rng.roll_d6();
            assert!((1..=6).contains(&v));
        }
    }

    // ── Xoshiro parity RNG ────────────────────────────────────────────────

    #[test]
    fn xoshiro_deterministic() {
        let mut a = Xoshiro256StarStar::new(42);
        let mut b = Xoshiro256StarStar::new(42);
        for _ in 0..1000 {
            assert_eq!(a.roll_d6(), b.roll_d6());
        }
    }

    #[test]
    fn xoshiro_d6_range() {
        let mut rng = Xoshiro256StarStar::new(7);
        for _ in 0..10_000 {
            let v = rng.roll_d6();
            assert!((1..=6).contains(&v));
        }
    }

    #[test]
    fn xoshiro_d6_uniformity() {
        let mut rng = Xoshiro256StarStar::new(555);
        let n = 60_000usize;
        let mut counts = [0usize; 6];
        for _ in 0..n {
            counts[(rng.roll_d6() - 1) as usize] += 1;
        }
        let expected = n as f64 / 6.0;
        let chi_sq: f64 = counts.iter()
            .map(|&c| (c as f64 - expected).powi(2) / expected)
            .sum();
        assert!(chi_sq < 11.07, "xoshiro chi-squared = {chi_sq:.3}");
    }
}
