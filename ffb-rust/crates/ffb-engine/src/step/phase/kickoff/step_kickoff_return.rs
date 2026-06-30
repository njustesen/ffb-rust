// TODO: full implementation. Stub placeholder for TRANSLATION_TRACKER.md.
pub struct StepKickoffReturn;

impl StepKickoffReturn {
    pub fn new() -> Self { Self }
}

impl Default for StepKickoffReturn {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_instance() {
        let _step = StepKickoffReturn::new();
    }

    #[test]
    fn default_works() {
        let _step = StepKickoffReturn::default();
    }

    #[test]
    fn size_is_zero() {
        assert_eq!(std::mem::size_of::<StepKickoffReturn>(), 0);
    }
}
