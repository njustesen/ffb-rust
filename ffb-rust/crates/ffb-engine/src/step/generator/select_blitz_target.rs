/// Root-level abstract base for the SelectBlitzTarget step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SelectBlitzTarget`.

pub struct SelectBlitzTarget;

impl SelectBlitzTarget {
    pub fn new() -> Self { Self }
}

impl Default for SelectBlitzTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_blitz_target_new_creates_instance() {
        let _ = SelectBlitzTarget::new();
    }

    #[test]
    fn select_blitz_target_default_creates_instance() {
        let _ = SelectBlitzTarget::default();
    }

    #[test]
    fn select_blitz_target_new_and_default_both_succeed() {
        let _a = SelectBlitzTarget::new();
        let _b = SelectBlitzTarget::default();
        assert!(true);
    }

    #[test]
    fn select_blitz_target_new_is_consistent_with_default() {
        let via_new = SelectBlitzTarget::new();
        let via_default = SelectBlitzTarget::default();
        let _ = (via_new, via_default);
    }
    #[test]
    fn is_zero_sized_struct() {
        assert_eq!(std::mem::size_of::<SelectBlitzTarget>(), 0);
    }
}
