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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_non_empty_returns_some() {
        assert!(GameOptionIdFactory::default().for_name("someOption").is_some());
    }

    #[test]
    fn for_name_empty_returns_none() {
        assert_eq!(GameOptionIdFactory::default().for_name(""), None);
    }
}
