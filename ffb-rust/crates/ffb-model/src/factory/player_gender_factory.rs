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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_gender() {
        assert!(PlayerGenderFactory::default().for_name("male").is_some());
        assert!(PlayerGenderFactory::default().for_name("female").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PlayerGenderFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = PlayerGenderFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(PlayerGenderFactory::default().for_name("nonbinary"), Some(PlayerGender::Nonbinary));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(PlayerGenderFactory::default().for_name(""), None);
    }
}
