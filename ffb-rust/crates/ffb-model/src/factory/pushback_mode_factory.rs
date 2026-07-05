use crate::model::PushbackMode;

/// 1:1 translation of com.fumbbl.ffb.factory.PushbackModeFactory.
pub struct PushbackModeFactory;

impl Default for PushbackModeFactory {
    fn default() -> Self { PushbackModeFactory }
}

impl PushbackModeFactory {
    pub fn for_name(&self, name: &str) -> Option<PushbackMode> {
        PushbackMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        let f = PushbackModeFactory::default();
        assert_eq!(f.for_name("regular"), Some(PushbackMode::REGULAR));
        assert_eq!(f.for_name("sideStep"), Some(PushbackMode::SIDE_STEP));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PushbackModeFactory::default().for_name("invalid"), None);
    }
}
