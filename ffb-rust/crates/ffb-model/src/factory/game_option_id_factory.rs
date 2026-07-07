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

    #[test]
    fn initialize_does_not_panic() {
        let mut f = GameOptionIdFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        // GameOptionId wraps any non-empty string, so a second non-empty name is valid
        assert!(GameOptionIdFactory::default().for_name("anotherOption").is_some());
    }

    #[test]
    fn for_name_whitespace_only_is_non_empty_and_returns_some() {
        // The factory only rejects truly empty strings; whitespace is treated as valid
        assert!(GameOptionIdFactory::default().for_name(" ").is_some());
    }
}
