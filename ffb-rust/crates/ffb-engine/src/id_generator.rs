/// 1:1 translation of `com.fumbbl.ffb.server.IdGenerator`.
///
/// Sequential ID generator — thread-safe increment in Java, uses `i64` field.
pub struct IdGenerator {
    last_id: i64,
}

impl IdGenerator {
    pub fn new(last_id: i64) -> Self {
        IdGenerator { last_id }
    }

    pub fn generate_id(&mut self) -> i64 {
        self.last_id += 1;
        self.last_id
    }

    pub fn last_id(&self) -> i64 {
        self.last_id
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        IdGenerator::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_id_increments_from_initial() {
        let mut gen = IdGenerator::new(0);
        assert_eq!(gen.generate_id(), 1);
        assert_eq!(gen.generate_id(), 2);
    }

    #[test]
    fn last_id_returns_current_without_incrementing() {
        let mut gen = IdGenerator::new(5);
        assert_eq!(gen.last_id(), 5);
        gen.generate_id();
        assert_eq!(gen.last_id(), 6);
    }

    #[test]
    fn generate_id_with_nonzero_start() {
        let mut gen = IdGenerator::new(100);
        assert_eq!(gen.generate_id(), 101);
    }

    #[test]
    fn default_starts_at_zero() {
        let mut gen = IdGenerator::default();
        assert_eq!(gen.last_id(), 0);
        assert_eq!(gen.generate_id(), 1);
    }

    #[test]
    fn negative_start_increments_correctly() {
        let mut gen = IdGenerator::new(-3);
        assert_eq!(gen.generate_id(), -2);
        assert_eq!(gen.generate_id(), -1);
        assert_eq!(gen.generate_id(), 0);
    }

    #[test]
    fn many_sequential_ids_are_strictly_increasing() {
        let mut gen = IdGenerator::new(0);
        let ids: Vec<i64> = (0..10).map(|_| gen.generate_id()).collect();
        for w in ids.windows(2) {
            assert_eq!(w[1], w[0] + 1);
        }
    }

    #[test]
    fn last_id_tracks_generate_id() {
        let mut gen = IdGenerator::new(0);
        for expected in 1..=5i64 {
            gen.generate_id();
            assert_eq!(gen.last_id(), expected);
        }
    }
}
