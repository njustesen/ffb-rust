/// Root-level abstract base for the QuickBite step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.QuickBite`.

pub struct QuickBite;

impl QuickBite {
    pub fn new() -> Self { Self }
}

impl Default for QuickBite {
    fn default() -> Self { Self::new() }
}
