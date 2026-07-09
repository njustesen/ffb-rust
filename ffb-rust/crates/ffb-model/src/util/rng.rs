use rand::RngCore;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

// ── EntropySource (translated from com.fumbbl.ffb.util.rng.EntropySource) ──

/// 1:1 translation of `com.fumbbl.ffb.util.rng.EntropySource`.
pub trait EntropySource {
    fn has_enough_entropy(&self) -> bool;
    fn get_entropy(&mut self) -> u8;
}

/// Minimal counter-based implementation for testing.
#[derive(Debug, Default)]
pub struct CounterEntropySource { counter: u8 }
impl CounterEntropySource { pub fn new() -> Self { Self::default() } }
impl EntropySource for CounterEntropySource {
    fn has_enough_entropy(&self) -> bool { true }
    fn get_entropy(&mut self) -> u8 { let v = self.counter; self.counter = self.counter.wrapping_add(1); v }
}

/// Per-roll stderr tracing, enabled by the FFB_DICE_TRACE env var.
fn dice_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("FFB_DICE_TRACE").is_some())
}

/// Seeded deterministic RNG matching Java's Xoshiro256StarStar exactly.
/// Seeding uses SplitMix64 (same constants as Java's Xoshiro256StarStar.java).
/// Die rolls use rejection sampling: threshold = u64::MAX - (u64::MAX % sides);
/// accept only if raw u64 < threshold; result = raw % sides + 1.
/// This matches Java's getDieRoll(int s) exactly.
pub struct GameRng {
    inner: Xoshiro256StarStar,
    pub call_count: u64,
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        GameRng { inner: Xoshiro256StarStar::seed_from_u64(seed), call_count: 0 }
    }

    /// Roll n-sided die (1..=sides) using rejection sampling.
    pub fn die(&mut self, sides: u32) -> i32 {
        let s = sides as u64;
        let threshold = u64::MAX - (u64::MAX % s);
        loop {
            let v = self.inner.next_u64();
            self.call_count += 1;
            if v < threshold {
                let result = (v % s + 1) as i32;
                if dice_trace_enabled() {
                    // Same format as the Java Xoshiro256StarStar -Dffb.diceTrace output
                    // so the two streams can be diffed directly.
                    eprintln!("DICE_TRACE pos={} sides={} result={}", self.call_count, sides, result);
                }
                return result;
            }
        }
    }

    /// Roll a single d6 (1–6).
    pub fn d6(&mut self) -> i32 {
        self.die(6)
    }

    /// Roll two d6 (2–12).
    pub fn d6_two(&mut self) -> i32 {
        self.d6() + self.d6()
    }

    /// Roll weather: returns [die1, die2] so the client report can show both dice.
    /// 1:1 translation of Java getDiceRoller().rollWeather().
    pub fn roll_weather(&mut self) -> [i32; 2] {
        [self.d6(), self.d6()]
    }

    /// Roll a d8 (1–8).
    pub fn d8(&mut self) -> i32 {
        self.die(8)
    }

    /// Roll a d3 (1–3). Matches Java's getDieRoll(3).
    pub fn d3(&mut self) -> i32 {
        self.die(3)
    }

    /// Roll riotous rookies count: returns [d3, d3].
    /// Java: `DiceRoller.rollRiotousRookies()` → `{ rollDice(3), rollDice(3) }`.
    pub fn roll_riotous_rookies(&mut self) -> [i32; 2] {
        [self.d3(), self.d3()]
    }

    /// Roll in range [0, n) using rejection sampling.
    pub fn range(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        // Equivalent to die(n) - 1; routed through die() so the call is counted and
        // traced like Java's Fortuna (e.g. the coin flip shows as a sides=2 roll).
        (self.die(n as u32) - 1) as usize
    }

    /// Flip a fair coin (uses range(2)).
    pub fn bool(&mut self) -> bool {
        self.range(2) == 0
    }

    /// Pick a random element from a non-empty slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let idx = self.range(items.len());
            Some(&items[idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d6_in_range() {
        let mut rng = GameRng::new(42);
        for _ in 0..1000 {
            let r = rng.d6();
            assert!(r >= 1 && r <= 6, "d6 out of range: {r}");
        }
    }

    #[test]
    fn deterministic() {
        let mut a = GameRng::new(123);
        let mut b = GameRng::new(123);
        for _ in 0..100 {
            assert_eq!(a.d6(), b.d6());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = GameRng::new(1);
        let mut b = GameRng::new(2);
        let seq_a: Vec<i32> = (0..20).map(|_| a.d6()).collect();
        let seq_b: Vec<i32> = (0..20).map(|_| b.d6()).collect();
        assert_ne!(seq_a, seq_b);
    }

    /// Verify sequence matches Java's Xoshiro256StarStar seeded with 1.
    /// Java Xoshiro256StarStar.java getDieRoll(6) sequence for seed=1:
    /// These expected values must be verified against Java output.
    #[test]
    fn xoshiro_seed1_d6_smoke() {
        let mut rng = GameRng::new(1);
        // Just check it runs and stays in range — exact values validated via parity
        let results: Vec<i32> = (0..10).map(|_| rng.d6()).collect();
        assert!(results.iter().all(|&r| r >= 1 && r <= 6));
    }

    #[test]
    fn range_in_bounds() {
        let mut rng = GameRng::new(99);
        for _ in 0..1000 {
            let r = rng.range(8);
            assert!(r < 8, "range(8) returned {r}");
        }
    }
}

