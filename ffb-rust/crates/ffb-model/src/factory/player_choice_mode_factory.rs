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
