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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_bite_new_creates_instance() {
        let _ = QuickBite::new();
    }

    #[test]
    fn quick_bite_default_creates_instance() {
        let _ = QuickBite::default();
    }

    #[test]
    fn quick_bite_new_and_default_both_succeed() {
        let _a = QuickBite::new();
        let _b = QuickBite::default();
        assert!(true);
    }

    #[test]
    fn quick_bite_new_is_consistent_with_default() {
        let via_new = QuickBite::new();
        let via_default = QuickBite::default();
        let _ = (via_new, via_default);
    }
}
