/// Root-level abstract base for the Punt step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.Punt`.

pub struct Punt;

impl Punt {
    pub fn new() -> Self { Self }
}

impl Default for Punt {
    fn default() -> Self { Self::new() }
}
