use crate::enums::PlayerAction;

/// 1:1 translation of com.fumbbl.ffb.factory.PlayerActionFactory.
pub struct PlayerActionFactory;

impl Default for PlayerActionFactory {
    fn default() -> Self { Self }
}

impl PlayerActionFactory {
    pub fn for_name(&self, name: &str) -> Option<PlayerAction> {
        PlayerAction::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
