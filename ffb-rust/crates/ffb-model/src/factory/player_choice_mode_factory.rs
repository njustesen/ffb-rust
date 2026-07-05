use crate::model::PlayerChoiceMode;

/// 1:1 translation of com.fumbbl.ffb.factory.PlayerChoiceModeFactory.
pub struct PlayerChoiceModeFactory;

impl Default for PlayerChoiceModeFactory {
    fn default() -> Self { PlayerChoiceModeFactory }
}

impl PlayerChoiceModeFactory {
    pub fn for_name(&self, name: &str) -> Option<PlayerChoiceMode> {
        PlayerChoiceMode::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_mode() {
        let f = PlayerChoiceModeFactory::default();
        assert_eq!(f.for_name("tentacles"), Some(PlayerChoiceMode::TENTACLES));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PlayerChoiceModeFactory::default().for_name("invalid"), None);
    }
}
