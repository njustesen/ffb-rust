use serde::{Deserialize, Serialize};
use crate::model::skill_def::SkillId;

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

    /// Java: `factory.forStatKey(key)` — returns the SkillId representing a +1 in this stat.
    pub fn skill_id_for_increase(self) -> SkillId {
        match self {
            PlayerStatKey::Ma => SkillId::MovementIncrease,
            PlayerStatKey::St => SkillId::StrengthIncrease,
            PlayerStatKey::Ag => SkillId::AgilityIncrease,
            PlayerStatKey::Pa => SkillId::PassingIncrease,
            PlayerStatKey::Av => SkillId::ArmourIncrease,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_has_five_entries() {
        assert_eq!(PlayerStatKey::all().len(), 5);
    }

    #[test]
    fn name_returns_correct_abbreviations() {
        assert_eq!(PlayerStatKey::Ma.name(), "MA");
        assert_eq!(PlayerStatKey::St.name(), "ST");
        assert_eq!(PlayerStatKey::Ag.name(), "AG");
        assert_eq!(PlayerStatKey::Pa.name(), "PA");
        assert_eq!(PlayerStatKey::Av.name(), "AV");
    }
}
