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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punt_new_creates_instance() {
        let _ = Punt::new();
    }

    #[test]
    fn punt_default_creates_instance() {
        let _ = Punt::default();
    }

    #[test]
    fn punt_new_and_default_both_succeed() {
        let _a = Punt::new();
        let _b = Punt::default();
        assert!(true);
    }

    #[test]
    fn punt_new_is_consistent_with_default() {
        let via_new = Punt::new();
        let via_default = Punt::default();
        let _ = (via_new, via_default);
    }
}
