use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.PlayerStatus.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlayerStatus {
    #[default]
    ACTIVE,
    JOURNEYMAN,
}

impl PlayerStatus {
    pub fn get_name(self) -> &'static str {
        match self {
            PlayerStatus::ACTIVE => "active",
            PlayerStatus::JOURNEYMAN => "journeyman",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "ACTIVE" => Some(PlayerStatus::ACTIVE),
            "JOURNEYMAN" => Some(PlayerStatus::JOURNEYMAN),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_active() {
        assert_eq!(PlayerStatus::for_name("ACTIVE"), Some(PlayerStatus::ACTIVE));
        assert_eq!(PlayerStatus::for_name("active"), Some(PlayerStatus::ACTIVE));
    }

    #[test]
    fn for_name_unknown() {
        assert_eq!(PlayerStatus::for_name("invalid"), None);
    }

    #[test]
    fn default_is_active() {
        assert_eq!(PlayerStatus::default(), PlayerStatus::ACTIVE);
    }

    #[test]
    fn get_name_returns_lowercase() {
        assert_eq!(PlayerStatus::ACTIVE.get_name(), "active");
        assert_eq!(PlayerStatus::JOURNEYMAN.get_name(), "journeyman");
    }

    #[test]
    fn serde_round_trip() {
        for v in [PlayerStatus::ACTIVE, PlayerStatus::JOURNEYMAN] {
            let s = serde_json::to_string(&v).unwrap();
            let back: PlayerStatus = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back);
        }
    }
}
