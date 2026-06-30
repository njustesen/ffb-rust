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
}
