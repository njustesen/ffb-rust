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
}
