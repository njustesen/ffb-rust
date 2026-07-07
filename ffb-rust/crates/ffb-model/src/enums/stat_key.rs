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

    #[test]
    fn skill_id_for_increase_covers_all_variants() {
        // Each stat key must map to a distinct skill id
        let ids: Vec<_> = PlayerStatKey::all().iter().map(|k| k.skill_id_for_increase()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), PlayerStatKey::all().len());
    }

    #[test]
    fn av_skill_id_for_increase_is_armour_increase() {
        assert_eq!(PlayerStatKey::Av.skill_id_for_increase(), SkillId::ArmourIncrease);
    }

    #[test]
    fn pa_skill_id_for_increase_is_passing_increase() {
        assert_eq!(PlayerStatKey::Pa.skill_id_for_increase(), SkillId::PassingIncrease);
    }

    #[test]
    fn ma_skill_id_for_increase_is_movement_increase() {
        assert_eq!(PlayerStatKey::Ma.skill_id_for_increase(), SkillId::MovementIncrease);
    }

    #[test]
    fn serde_round_trip() {
        let k = PlayerStatKey::St;
        let json = serde_json::to_string(&k).unwrap();
        let back: PlayerStatKey = serde_json::from_str(&json).unwrap();
        assert_eq!(k, back);
    }
}
