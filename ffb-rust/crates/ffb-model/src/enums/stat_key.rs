use serde::{Deserialize, Serialize};

/// Player stat identifiers — mirrors Java's `PlayerStatKey` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerStatKey {
    Ma,
    St,
    Ag,
    Pa,
    Av,
}

impl PlayerStatKey {
    pub fn name(self) -> &'static str {
        match self {
            PlayerStatKey::Ma => "MA",
            PlayerStatKey::St => "ST",
            PlayerStatKey::Ag => "AG",
            PlayerStatKey::Pa => "PA",
            PlayerStatKey::Av => "AV",
        }
    }

    pub fn all() -> &'static [PlayerStatKey] {
        &[
            PlayerStatKey::Ma,
            PlayerStatKey::St,
            PlayerStatKey::Ag,
            PlayerStatKey::Pa,
            PlayerStatKey::Av,
        ]
    }
}
