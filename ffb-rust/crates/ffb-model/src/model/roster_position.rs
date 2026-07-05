use serde::{Deserialize, Serialize};
use crate::enums::{PlayerType, PlayerGender, SkillCategory};
use crate::model::skill_def::SkillWithValue;

/// A position template within a team roster.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RosterPosition {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub shorthand: Option<String>,
    pub player_type: PlayerType,
    pub gender: PlayerGender,
    /// Maximum number of this position allowed in a team.
    pub quantity: i32,
    pub cost: i32,

    pub movement: i32,
    pub strength: i32,
    pub agility: i32,
    pub passing: i32,
    pub armour: i32,

    /// Starting skills.
    pub skills: Vec<SkillWithValue>,
    /// Skill categories available on a normal (single) roll.
    pub skill_categories_normal: Vec<SkillCategory>,
    /// Skill categories available on a double roll.
    pub skill_categories_double: Vec<SkillCategory>,

    pub keywords: Vec<String>,

    pub is_big_guy: bool,
    pub is_undead: bool,
    pub is_thrall: bool,
    pub race: Option<String>,
    /// ID of the position this replaces (star player slot, etc.).
    pub replaces_position: Option<String>,
}

impl RosterPosition {
    pub fn is_star_player(&self) -> bool {
        self.player_type == PlayerType::Star
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender, SkillCategory};

    fn minimal() -> RosterPosition {
        RosterPosition {
            id: "lineman".into(),
            name: "Lineman".into(),
            display_name: None,
            shorthand: None,
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            quantity: 12,
            cost: 50_000,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            skills: vec![],
            skill_categories_normal: vec![SkillCategory::General],
            skill_categories_double: vec![
                SkillCategory::Agility,
                SkillCategory::Strength,
                SkillCategory::Passing,
            ],
            keywords: vec![],
            is_big_guy: false,
            is_undead: false,
            is_thrall: false,
            race: None,
            replaces_position: None,
        }
    }

    #[test]
    fn serde_round_trip() {
        let pos = minimal();
        let json = serde_json::to_string(&pos).unwrap();
        let back: RosterPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(pos.id, back.id);
        assert_eq!(pos.movement, back.movement);
    }

    #[test]
    fn is_star_player_false_for_regular() {
        assert!(!minimal().is_star_player());
    }

    #[test]
    fn is_star_player_true_for_star_type() {
        let mut pos = minimal();
        pos.player_type = PlayerType::Star;
        assert!(pos.is_star_player());
    }
}
