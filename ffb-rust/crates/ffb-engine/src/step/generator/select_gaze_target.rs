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
