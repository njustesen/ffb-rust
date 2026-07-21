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
    fn from_name_round_trips() {
        assert_eq!(ConcedeGameStatus::from_name("requested"), Some(ConcedeGameStatus::REQUESTED));
        assert_eq!(ConcedeGameStatus::from_name("invalid"), None);
    }

    #[test]
    fn all_names_round_trip() {
        for v in [ConcedeGameStatus::REQUESTED, ConcedeGameStatus::CONFIRMED, ConcedeGameStatus::DENIED] {
            assert_eq!(ConcedeGameStatus::from_name(v.get_name()), Some(v));
        }
    }

    #[test]
    fn serde_round_trip() {
        let s = serde_json::to_string(&ConcedeGameStatus::CONFIRMED).unwrap();
        let back: ConcedeGameStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(back, ConcedeGameStatus::CONFIRMED);
    }

}
