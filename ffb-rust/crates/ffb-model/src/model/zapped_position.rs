use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use crate::model::skill_def::SkillWithValue;

/// Java: ZappedPosition — wraps a RosterPosition and provides fixed "frog" stats + skills.
/// Stat values are edition-specific (from GameMechanic.zappedPlayerStats()).
/// Skills are always: Dodge, NoHands, Titchy, Stunty, VeryLongLegs, Leap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZappedPosition {
    pub original_position_id: String,
    pub original_position_name: String,
    pub movement: i32,
    pub strength: i32,
    pub agility: i32,
    pub passing: i32,
    pub armour: i32,
}

impl ZappedPosition {
    pub fn new(
        original_position_id: String,
        original_position_name: String,
        movement: i32,
        strength: i32,
        agility: i32,
        passing: i32,
        armour: i32,
    ) -> Self {
        Self { original_position_id, original_position_name, movement, strength, agility, passing, armour }
    }

    /// BB2016 zapped stats: MA=5, ST=1, AG=4, PA=0, AV=4.
    pub fn new_bb2016(original_position_id: String, original_position_name: String) -> Self {
        Self::new(original_position_id, original_position_name, 5, 1, 4, 0, 4)
    }

    /// BB2020/BB2025 zapped stats: MA=5, ST=1, AG=2, PA=0, AV=5.
    pub fn new_bb2020(original_position_id: String, original_position_name: String) -> Self {
        Self::new(original_position_id, original_position_name, 5, 1, 2, 0, 5)
    }

    pub fn get_movement(&self) -> i32 { self.movement }
    pub fn get_strength(&self) -> i32 { self.strength }
    pub fn get_agility(&self) -> i32 { self.agility }
    pub fn get_passing(&self) -> i32 { self.passing }
    pub fn get_armour(&self) -> i32 { self.armour }
    pub fn get_id(&self) -> &str { &self.original_position_id }
    pub fn get_name(&self) -> &str { &self.original_position_name }
    pub fn get_race(&self) -> &str { "Transmogrified Frog" }
    pub fn get_shorthand(&self) -> &str { "zf" }

    /// Fixed skill set for all zapped players (Java: ZappedPosition constructor).
    pub fn get_skills() -> Vec<SkillWithValue> {
        vec![
            SkillWithValue { skill_id: SkillId::Dodge, value: None },
            SkillWithValue { skill_id: SkillId::NoHands, value: None },
            SkillWithValue { skill_id: SkillId::Titchy, value: None },
            SkillWithValue { skill_id: SkillId::Stunty, value: None },
            SkillWithValue { skill_id: SkillId::VeryLongLegs, value: None },
            SkillWithValue { skill_id: SkillId::Leap, value: None },
        ]
    }
}

impl Default for ZappedPosition {
    fn default() -> Self {
        Self::new_bb2020(String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2016_stats() {
        let pos = ZappedPosition::new_bb2016("lineman".into(), "Lineman".into());
        assert_eq!(pos.get_movement(), 5);
        assert_eq!(pos.get_strength(), 1);
        assert_eq!(pos.get_agility(), 4);
        assert_eq!(pos.get_passing(), 0);
        assert_eq!(pos.get_armour(), 4);
    }

    #[test]
    fn bb2020_stats() {
        let pos = ZappedPosition::new_bb2020("lineman".into(), "Lineman".into());
        assert_eq!(pos.get_movement(), 5);
        assert_eq!(pos.get_strength(), 1);
        assert_eq!(pos.get_agility(), 2);
        assert_eq!(pos.get_passing(), 0);
        assert_eq!(pos.get_armour(), 5);
    }

    #[test]
    fn get_skills_has_six_entries() {
        let skills = ZappedPosition::get_skills();
        assert_eq!(skills.len(), 6);
    }

    #[test]
    fn get_skills_contains_dodge() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::Dodge));
    }

    #[test]
    fn get_skills_contains_no_hands() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::NoHands));
    }

    #[test]
    fn get_skills_contains_stunty() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::Stunty));
    }

    #[test]
    fn get_skills_contains_titchy() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::Titchy));
    }

    #[test]
    fn get_skills_contains_very_long_legs() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::VeryLongLegs));
    }

    #[test]
    fn get_skills_contains_leap() {
        let skills = ZappedPosition::get_skills();
        assert!(skills.iter().any(|s| s.skill_id == SkillId::Leap));
    }

    #[test]
    fn race_is_transmogrified_frog() {
        let pos = ZappedPosition::new_bb2020("p1".into(), "Player".into());
        assert_eq!(pos.get_race(), "Transmogrified Frog");
    }

    #[test]
    fn shorthand_is_zf() {
        let pos = ZappedPosition::new_bb2020("p1".into(), "Player".into());
        assert_eq!(pos.get_shorthand(), "zf");
    }

    #[test]
    fn delegates_id_to_original() {
        let pos = ZappedPosition::new_bb2016("lineman-id".into(), "Lineman".into());
        assert_eq!(pos.get_id(), "lineman-id");
    }

    #[test]
    fn delegates_name_to_original() {
        let pos = ZappedPosition::new_bb2016("p1".into(), "Snotling".into());
        assert_eq!(pos.get_name(), "Snotling");
    }
}
