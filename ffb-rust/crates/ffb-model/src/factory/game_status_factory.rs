use crate::enums::GameStatus;

/// 1:1 translation of com.fumbbl.ffb.factory.GameStatusFactory.
pub struct GameStatusFactory;

impl Default for GameStatusFactory {
    fn default() -> Self { Self }
}

impl GameStatusFactory {
    pub fn for_name(&self, name: &str) -> Option<GameStatus> {
        GameStatus::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
