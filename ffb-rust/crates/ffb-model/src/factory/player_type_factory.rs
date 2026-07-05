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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_type() {
        assert_eq!(PlayerTypeFactory::default().for_name("Regular"), Some(PlayerType::Regular));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PlayerTypeFactory::default().for_name("invalid"), None);
    }
}
