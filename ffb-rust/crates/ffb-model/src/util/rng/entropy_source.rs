/// 1:1 translation of `com.fumbbl.ffb.util.rng.EntropySource`.
///
/// The trait and `CounterEntropySource` implementation live in `util/rng.rs`
/// (alongside `GameRng`) to avoid the Rust limitation of not allowing both a
/// `rng.rs` file and a `rng/` directory.  This file re-exports them for
/// ergonomic imports from the `rng/` subpath.
pub use crate::util::rng::{EntropySource, CounterEntropySource};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_has_entropy() {
        let src = CounterEntropySource::new();
        assert!(src.has_enough_entropy());
    }

    #[test]
    fn counter_increments() {
        let mut src = CounterEntropySource::new();
        assert_eq!(src.get_entropy(), 0);
        assert_eq!(src.get_entropy(), 1);
    }
}
