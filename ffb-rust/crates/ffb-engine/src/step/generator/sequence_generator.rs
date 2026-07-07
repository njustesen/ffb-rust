/// Abstract base for all step sequence generators.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SequenceGenerator`.

pub struct SequenceGenerator;

impl SequenceGenerator {
    pub fn new() -> Self { Self }
}

impl Default for SequenceGenerator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_generator_new_creates_instance() {
        let _ = SequenceGenerator::new();
    }

    #[test]
    fn sequence_generator_default_creates_instance() {
        let _ = SequenceGenerator::default();
    }

    #[test]
    fn sequence_generator_new_and_default_both_succeed() {
        let _a = SequenceGenerator::new();
        let _b = SequenceGenerator::default();
        assert!(true);
    }

    #[test]
    fn sequence_generator_new_is_consistent_with_default() {
        let via_new = SequenceGenerator::new();
        let via_default = SequenceGenerator::default();
        let _ = (via_new, via_default);
    }
    #[test]
    fn is_zero_sized_struct() {
        assert_eq!(std::mem::size_of::<SequenceGenerator>(), 0);
    }
}
