use crate::model::game_options::GameOptionId;

/// 1:1 translation of com.fumbbl.ffb.factory.GameOptionIdFactory.
pub struct GameOptionIdFactory;

impl Default for GameOptionIdFactory {
    fn default() -> Self { GameOptionIdFactory }
}

impl GameOptionIdFactory {
    pub fn for_name(&self, name: &str) -> Option<GameOptionId> {
        if name.is_empty() { None } else { Some(GameOptionId::new(name)) }
    }

    pub fn initialize(&mut self) {}
}
