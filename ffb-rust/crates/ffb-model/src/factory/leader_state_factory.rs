use crate::enums::LeaderState;

/// 1:1 translation of com.fumbbl.ffb.factory.LeaderStateFactory.
pub struct LeaderStateFactory;

impl Default for LeaderStateFactory {
    fn default() -> Self { Self }
}

impl LeaderStateFactory {
    pub fn for_name(&self, name: &str) -> Option<LeaderState> {
        LeaderState::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_state() {
        assert_eq!(LeaderStateFactory::default().for_name("available"), Some(LeaderState::Available));
        assert_eq!(LeaderStateFactory::default().for_name("used"), Some(LeaderState::Used));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(LeaderStateFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = LeaderStateFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(LeaderStateFactory::default().for_name("none"), Some(LeaderState::None));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(LeaderStateFactory::default().for_name(""), None);
    }
}
