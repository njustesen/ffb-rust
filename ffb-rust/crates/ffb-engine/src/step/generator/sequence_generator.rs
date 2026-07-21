/// Abstract base for all step sequence generators.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SequenceGenerator`.

pub struct SequenceGenerator;

impl SequenceGenerator {
    pub fn new() -> Self { Self }
}

impl Default for SequenceGenerator {
    fn default() -> Self { Self::new() }
}
