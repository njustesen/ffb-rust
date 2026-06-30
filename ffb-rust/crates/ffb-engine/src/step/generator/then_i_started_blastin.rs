/// Root-level abstract base for the ThenIStartedBlastin step sequence generator.
/// No inner SequenceParams — uses base SequenceGenerator.SequenceParams.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.ThenIStartedBlastin`.

pub struct ThenIStartedBlastin;

impl ThenIStartedBlastin {
    pub fn new() -> Self { Self }
}

impl Default for ThenIStartedBlastin {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn then_i_started_blastin_new_creates_instance() {
        let _ = ThenIStartedBlastin::new();
    }

    #[test]
    fn then_i_started_blastin_default_creates_instance() {
        let _ = ThenIStartedBlastin::default();
    }
}
