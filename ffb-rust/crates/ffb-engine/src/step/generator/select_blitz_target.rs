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
}
