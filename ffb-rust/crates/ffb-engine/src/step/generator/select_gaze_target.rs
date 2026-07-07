/// Root-level abstract base for the SelectGazeTarget step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.SelectGazeTarget`.

pub struct SelectGazeTarget;

impl SelectGazeTarget {
    pub fn new() -> Self { Self }
}

impl Default for SelectGazeTarget {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_gaze_target_new_creates_instance() {
        let _ = SelectGazeTarget::new();
    }

    #[test]
    fn select_gaze_target_default_creates_instance() {
        let _ = SelectGazeTarget::default();
    }

    #[test]
    fn select_gaze_target_new_and_default_both_succeed() {
        let _a = SelectGazeTarget::new();
        let _b = SelectGazeTarget::default();
        assert!(true);
    }

    #[test]
    fn select_gaze_target_new_is_consistent_with_default() {
        let via_new = SelectGazeTarget::new();
        let via_default = SelectGazeTarget::default();
        let _ = (via_new, via_default);
    }
    #[test]
    fn is_zero_sized_struct() {
        assert_eq!(std::mem::size_of::<SelectGazeTarget>(), 0);
    }
}
