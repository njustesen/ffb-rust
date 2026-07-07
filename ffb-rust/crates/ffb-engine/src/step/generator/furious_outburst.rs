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

    #[test]
    fn furious_outburst_new_and_default_both_succeed() {
        let _a = FuriousOutburst::new();
        let _b = FuriousOutburst::default();
        assert!(true);
    }

    #[test]
    fn furious_outburst_new_is_consistent_with_default() {
        // Both construction paths must be available without panic.
        let via_new = FuriousOutburst::new();
        let via_default = FuriousOutburst::default();
        let _ = (via_new, via_default);
    }
}
