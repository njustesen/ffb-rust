/// TODO: full implementation. Mirrors Java `com.fumbbl.ffb.server.step.mixed/pass/state/PassState`.
pub struct PassState;

impl PassState {
    pub fn new() -> Self { Self }
}

impl Default for PassState {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_instance() {
        let _state = PassState::new();
    }

    #[test]
    fn default_creates_instance() {
        let _state = PassState::default();
    }

    #[test]
    fn new_and_default_are_equivalent() {
        // PassState is a unit struct — both construction paths yield the same type.
        let _a = PassState::new();
        let _b = PassState::default();
    }
}
