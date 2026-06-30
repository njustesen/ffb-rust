/// Root-level abstract base for the FuriousOutburst step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.FuriousOutburst`.

pub struct FuriousOutburst;

impl FuriousOutburst {
    pub fn new() -> Self { Self }
}

impl Default for FuriousOutburst {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn furious_outburst_new_creates_instance() {
        let _ = FuriousOutburst::new();
    }

    #[test]
    fn furious_outburst_default_creates_instance() {
        let _ = FuriousOutburst::default();
    }
}
