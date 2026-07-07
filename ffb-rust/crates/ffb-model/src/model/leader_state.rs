use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.LeaderState.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaderState {
    NONE,
    AVAILABLE,
    USED,
}

impl LeaderState {
    pub fn get_name(self) -> &'static str {
        match self {
            LeaderState::NONE => "none",
            LeaderState::AVAILABLE => "available",
            LeaderState::USED => "used",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(LeaderState::NONE),
            "available" => Some(LeaderState::AVAILABLE),
            "used" => Some(LeaderState::USED),
            _ => None,
        }
    }
}

impl Default for LeaderState {
    fn default() -> Self { LeaderState::NONE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_correct_variant() {
        assert_eq!(LeaderState::for_name("none"), Some(LeaderState::NONE));
        assert_eq!(LeaderState::for_name("available"), Some(LeaderState::AVAILABLE));
        assert_eq!(LeaderState::for_name("used"), Some(LeaderState::USED));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(LeaderState::for_name("AVAILABLE"), None);
        assert_eq!(LeaderState::for_name("invalid"), None);
    }

    #[test]
    fn default_is_none() {
        assert_eq!(LeaderState::default(), LeaderState::NONE);
    }

    #[test]
    fn get_name_matches_for_name_input() {
        for &state in &[LeaderState::NONE, LeaderState::AVAILABLE, LeaderState::USED] {
            assert_eq!(LeaderState::for_name(state.get_name()), Some(state));
        }
    }

    #[test]
    fn for_name_is_case_sensitive() {
        assert_eq!(LeaderState::for_name("None"), None);
        assert_eq!(LeaderState::for_name("USED"), None);
        assert_eq!(LeaderState::for_name("Available"), None);
    }
}
