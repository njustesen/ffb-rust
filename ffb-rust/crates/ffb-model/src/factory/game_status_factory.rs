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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_status() {
        assert_eq!(GameStatusFactory::default().for_name("active"), Some(GameStatus::Active));
        assert_eq!(GameStatusFactory::default().for_name("scheduled"), Some(GameStatus::Scheduled));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(GameStatusFactory::default().for_name("invalid"), None);
    }
}
