use crate::enums::PlayerType;

/// 1:1 translation of com.fumbbl.ffb.factory.PlayerTypeFactory.
pub struct PlayerTypeFactory;

impl Default for PlayerTypeFactory {
    fn default() -> Self { Self }
}

impl PlayerTypeFactory {
    pub fn for_name(&self, name: &str) -> Option<PlayerType> {
        PlayerType::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
