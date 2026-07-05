use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.ConcedeGameStatus.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcedeGameStatus {
    REQUESTED,
    CONFIRMED,
    DENIED,
}

impl ConcedeGameStatus {
    pub fn get_name(self) -> &'static str {
        match self {
            ConcedeGameStatus::REQUESTED => "requested",
            ConcedeGameStatus::CONFIRMED => "confirmed",
            ConcedeGameStatus::DENIED => "denied",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "requested" => Some(ConcedeGameStatus::REQUESTED),
            "confirmed" => Some(ConcedeGameStatus::CONFIRMED),
            "denied" => Some(ConcedeGameStatus::DENIED),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_returns_expected_strings() {
        assert_eq!(ConcedeGameStatus::REQUESTED.get_name(), "requested");
        assert_eq!(ConcedeGameStatus::CONFIRMED.get_name(), "confirmed");
        assert_eq!(ConcedeGameStatus::DENIED.get_name(), "denied");
    }

    #[test]
    fn from_name_round_trips() {
        assert_eq!(ConcedeGameStatus::from_name("requested"), Some(ConcedeGameStatus::REQUESTED));
        assert_eq!(ConcedeGameStatus::from_name("invalid"), None);
    }
}
