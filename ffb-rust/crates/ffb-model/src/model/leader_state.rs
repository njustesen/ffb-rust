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
