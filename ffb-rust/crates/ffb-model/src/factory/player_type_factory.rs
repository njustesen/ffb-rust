use crate::enums::PlayerType;

/// 1:1 translation of com.fumbbl.ffb.factory.PlayerTypeFactory.
pub struct PlayerTypeFactory;

impl Default for PlayerTypeFactory {
    fn default() -> Self { Self }
}

impl PlayerTypeFactory {
    /// Java: `forName` -- `equalsIgnoreCase` over `PlayerType.values()` in declaration order, then
    /// the legacy alias "Normal" -> REGULAR; null for an empty name.
    pub fn for_name(&self, name: &str) -> Option<PlayerType> {
        use crate::util::java_json::java_equals_ignore_case;
        if name.is_empty() {
            return None;
        }
        const VALUES: [PlayerType; 9] = [
            PlayerType::Regular, PlayerType::BigGuy, PlayerType::Star, PlayerType::Irregular,
            PlayerType::RiotousRookie, PlayerType::RaisedFromDead, PlayerType::Mercenary,
            PlayerType::PlagueRidden, PlayerType::InfamousStaff,
        ];
        if let Some(t) = VALUES.iter().copied().find(|t| java_equals_ignore_case(name, t.name())) {
            return Some(t);
        }
        if java_equals_ignore_case(name, "Normal") {
            return Some(PlayerType::Regular);
        }
        None
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

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(PlayerTypeFactory::default().for_name("Star"), Some(PlayerType::Star));
    }

    #[test]
    fn for_name_matches_java_ignore_case_and_the_normal_alias() {
        assert_eq!(PlayerTypeFactory::default().for_name("big guy"), Some(PlayerType::BigGuy));
        assert_eq!(PlayerTypeFactory::default().for_name("Normal"), Some(PlayerType::Regular));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(PlayerTypeFactory::default().for_name(""), None);
    }
}
