use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerType, PlayerGender, SeriousInjuryKind};
use crate::model::skill_def::{SkillId, SkillWithValue};
use crate::model::roster_position::RosterPosition;

/// Unique player identifier (string id as in the Java model).
pub type PlayerId = String;

/// A concrete player instance on a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub nr: i32,
    pub position_id: String,
    pub player_type: PlayerType,
    pub gender: PlayerGender,

    // Base stats (before modifiers)
    pub movement: i32,
    pub strength: i32,
    pub agility: i32,
    pub passing: i32,
    pub armour: i32,

    /// Skills the position starts with (defined on the roster position).
    #[serde(default)]
    pub starting_skills: Vec<SkillWithValue>,
    /// Skills gained via levelling (on top of position starting skills).
    pub extra_skills: Vec<SkillWithValue>,
    /// Skills granted temporarily (cards, prayers, etc.).
    pub temporary_skills: Vec<SkillWithValue>,
    /// Skills used this turn (reset at turn start).
    pub used_skills: HashSet<SkillId>,

    /// Permanent serious injuries reducing stats.
    pub niggling_injuries: i32,
    pub stat_injuries: Vec<SeriousInjuryKind>,

    pub current_spps: i32,
    pub career_spps: i32,

    /// Race identifier for Animosity checks (e.g. "Hobgoblin", "Bull Centaur").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub race: Option<String>,
}

impl Player {
    pub fn movement_with_modifiers(&self) -> i32 {
        self.movement
    }

    pub fn strength_with_modifiers(&self) -> i32 {
        self.strength
    }

    pub fn agility_with_modifiers(&self) -> i32 {
        self.agility
    }

    pub fn passing_with_modifiers(&self) -> i32 {
        self.passing
    }

    pub fn armour_with_modifiers(&self) -> i32 {
        self.armour
    }

    pub fn all_skill_ids(&self) -> impl Iterator<Item = SkillId> + '_ {
        self.starting_skills
            .iter()
            .chain(self.extra_skills.iter())
            .chain(self.temporary_skills.iter())
            .map(|sw| sw.skill_id)
    }

    pub fn has_skill(&self, id: SkillId) -> bool {
        self.all_skill_ids().any(|s| s == id)
    }

    /// Construct a new player instance from a roster position template.
    pub fn from_position(id: impl Into<String>, name: impl Into<String>, nr: i32, pos: &RosterPosition) -> Self {
        Player {
            id: id.into(),
            name: name.into(),
            nr,
            position_id: pos.id.clone(),
            player_type: pos.player_type,
            gender: pos.gender,
            movement: pos.movement,
            strength: pos.strength,
            agility: pos.agility,
            passing: pos.passing,
            armour: pos.armour,
            starting_skills: pos.skills.clone(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: pos.race.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender};

    fn test_player() -> Player {
        Player {
            id: "p1".into(),
            name: "Joe".into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        }
    }

    #[test]
    fn serde_round_trip() {
        let p = test_player();
        let json = serde_json::to_string(&p).unwrap();
        let back: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, back.id);
        assert_eq!(p.movement, back.movement);
    }

    #[test]
    fn has_skill_false_when_empty() {
        let p = test_player();
        assert!(!p.has_skill(SkillId::Block));
    }

    #[test]
    fn has_skill_true_for_starting_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::Block, value: None });
        assert!(p.has_skill(SkillId::Block));
        assert!(!p.has_skill(SkillId::Tackle));
    }

    #[test]
    fn has_skill_true_for_extra_skill() {
        use crate::model::skill_def::SkillWithValue;
        let mut p = test_player();
        p.extra_skills.push(SkillWithValue { skill_id: SkillId::Dodge, value: None });
        assert!(p.has_skill(SkillId::Dodge));
    }

    #[test]
    fn from_position_copies_starting_skills() {
        use crate::model::skill_def::SkillWithValue;
        use crate::model::roster_position::RosterPosition;
        use crate::enums::{PlayerType, PlayerGender, SkillCategory};
        let pos = RosterPosition {
            id: "blitzer".into(),
            name: "Blitzer".into(),
            display_name: None,
            shorthand: None,
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            quantity: 4,
            cost: 80_000,
            movement: 7,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 9,
            skills: vec![SkillWithValue { skill_id: SkillId::Block, value: None }],
            skill_categories_normal: vec![SkillCategory::General],
            skill_categories_double: vec![],
            keywords: vec![],
            is_big_guy: false,
            is_undead: false,
            is_thrall: false,
            race: None,
            replaces_position: None,
        };
        let p = Player::from_position("p1", "Blitzer Joe", 3, &pos);
        assert_eq!(p.position_id, "blitzer");
        assert_eq!(p.movement, 7);
        assert!(p.has_skill(SkillId::Block));
        assert!(!p.has_skill(SkillId::Tackle));
    }
}
