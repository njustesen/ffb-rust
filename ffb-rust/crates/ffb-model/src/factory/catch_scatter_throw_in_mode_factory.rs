use crate::model::CatchScatterThrowInMode;

/// 1:1 translation of com.fumbbl.ffb.factory.CatchScatterThrowInModeFactory.
pub struct CatchScatterThrowInModeFactory;

impl Default for CatchScatterThrowInModeFactory {
    fn default() -> Self { CatchScatterThrowInModeFactory }
}

impl CatchScatterThrowInModeFactory {
    pub fn for_name(&self, name: &str) -> Option<CatchScatterThrowInMode> {
        CatchScatterThrowInMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        assert_eq!(
            CatchScatterThrowInModeFactory::default().for_name("catchHandOff"),
            Some(CatchScatterThrowInMode::CatchHandOff)
        );
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(CatchScatterThrowInModeFactory::default().for_name("invalid"), None);
    }
}
