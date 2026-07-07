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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_action() {
        let f = PlayerActionFactory::default();
        assert_eq!(f.for_name("move"), Some(PlayerAction::Move));
        assert_eq!(f.for_name("block"), Some(PlayerAction::Block));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PlayerActionFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = PlayerActionFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(PlayerActionFactory::default().for_name("foul"), Some(PlayerAction::Foul));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(PlayerActionFactory::default().for_name(""), None);
    }
}
