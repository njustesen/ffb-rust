use crate::enums::PlayerGender;

/// 1:1 translation of com.fumbbl.ffb.factory.PlayerGenderFactory.
pub struct PlayerGenderFactory;

impl Default for PlayerGenderFactory {
    fn default() -> Self { Self }
}

impl PlayerGenderFactory {
    pub fn for_name(&self, name: &str) -> Option<PlayerGender> {
        PlayerGender::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
