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
}
