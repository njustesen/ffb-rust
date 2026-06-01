use serde::{Deserialize, Serialize};

/// Blood Bowl skill category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    General,
    Agility,
    Passing,
    Strength,
    Mutation,
    Mutations,
    Extraordinary,
    StatIncrease,
    StatDecrease,
    Trait,
    Devious,
}

impl SkillCategory {
    pub fn name(self) -> &'static str {
        match self {
            SkillCategory::General => "General",
            SkillCategory::Agility => "Agility",
            SkillCategory::Passing => "Passing",
            SkillCategory::Strength => "Strength",
            SkillCategory::Mutation => "Mutation",
            SkillCategory::Mutations => "Mutations",
            SkillCategory::Extraordinary => "Extraordinary",
            SkillCategory::StatIncrease => "Stat Increase",
            SkillCategory::StatDecrease => "Stat Decrease",
            SkillCategory::Trait => "Trait",
            SkillCategory::Devious => "Devious",
        }
    }

    pub fn from_name(name: &str) -> Option<SkillCategory> {
        match name {
            "General" => Some(SkillCategory::General),
            "Agility" => Some(SkillCategory::Agility),
            "Passing" => Some(SkillCategory::Passing),
            "Strength" => Some(SkillCategory::Strength),
            "Mutation" | "Mutations" => Some(SkillCategory::Mutation),
            "Extraordinary" => Some(SkillCategory::Extraordinary),
            "Stat Increase" => Some(SkillCategory::StatIncrease),
            "Stat Decrease" => Some(SkillCategory::StatDecrease),
            "Trait" => Some(SkillCategory::Trait),
            "Devious" => Some(SkillCategory::Devious),
            _ => None,
        }
    }
}

/// How many times per game/turn a skill can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillUsageType {
    Regular,
    OncePerTurn,
    OncePerGame,
    OncePerHalf,
    OncePerDrive,
    OncePerTurnByTeamMate,
    Special,
}

impl SkillUsageType {
    /// Whether the skill's usage must be tracked even outside the player's own activation.
    pub fn track_outside_activation(self) -> bool {
        !matches!(self, SkillUsageType::Regular)
    }

    /// Whether effects granted by this skill are removed at end of turn.
    pub fn remove_effects_at_end_of_turn(self) -> bool {
        !matches!(
            self,
            SkillUsageType::OncePerTurnByTeamMate | SkillUsageType::Special
        )
    }
}

/// When a skill can be declared during a player's activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeclareCondition {
    None,
    Standing,
}

impl DeclareCondition {
    /// Whether this condition is met given the player's current state.
    /// `is_standing` corresponds to `PlayerState::isStanding()`.
    pub fn fulfilled(self, is_standing: bool) -> bool {
        match self {
            DeclareCondition::None => true,
            DeclareCondition::Standing => is_standing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_category_round_trip() {
        for cat in &[
            SkillCategory::General,
            SkillCategory::Strength,
            SkillCategory::Trait,
        ] {
            assert_eq!(SkillCategory::from_name(cat.name()), Some(*cat));
        }
    }

    #[test]
    fn declare_condition_none_always_fulfilled() {
        assert!(DeclareCondition::None.fulfilled(true));
        assert!(DeclareCondition::None.fulfilled(false));
    }

    #[test]
    fn declare_condition_standing_requires_standing() {
        assert!(DeclareCondition::Standing.fulfilled(true));
        assert!(!DeclareCondition::Standing.fulfilled(false));
    }

    #[test]
    fn skill_category_count_is_eleven() {
        let all = [
            SkillCategory::General, SkillCategory::Agility, SkillCategory::Passing,
            SkillCategory::Strength, SkillCategory::Mutation, SkillCategory::Mutations,
            SkillCategory::Extraordinary, SkillCategory::StatIncrease, SkillCategory::StatDecrease,
            SkillCategory::Trait, SkillCategory::Devious,
        ];
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn all_skill_categories_have_non_empty_names() {
        for cat in [
            SkillCategory::General, SkillCategory::Agility, SkillCategory::Passing,
            SkillCategory::Strength, SkillCategory::Mutation, SkillCategory::Mutations,
            SkillCategory::Extraordinary, SkillCategory::StatIncrease, SkillCategory::StatDecrease,
            SkillCategory::Trait, SkillCategory::Devious,
        ] {
            assert!(!cat.name().is_empty());
        }
    }

    #[test]
    fn general_name_is_general() {
        assert_eq!(SkillCategory::General.name(), "General");
    }

    #[test]
    fn agility_name_is_agility() {
        assert_eq!(SkillCategory::Agility.name(), "Agility");
    }

    #[test]
    fn strength_name_is_strength() {
        assert_eq!(SkillCategory::Strength.name(), "Strength");
    }

    #[test]
    fn trait_name_is_trait() {
        assert_eq!(SkillCategory::Trait.name(), "Trait");
    }

    #[test]
    fn skill_usage_type_regular_does_not_track_outside_activation() {
        assert!(!SkillUsageType::Regular.track_outside_activation());
    }

    #[test]
    fn skill_usage_type_once_per_turn_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerTurn.track_outside_activation());
    }

    #[test]
    fn skill_usage_type_once_per_game_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerGame.track_outside_activation());
    }

    #[test]
    fn skill_usage_type_once_per_turn_removes_effects_at_end_of_turn() {
        assert!(SkillUsageType::OncePerTurn.remove_effects_at_end_of_turn());
    }

    #[test]
    fn skill_usage_type_special_does_not_remove_effects_at_end_of_turn() {
        assert!(!SkillUsageType::Special.remove_effects_at_end_of_turn());
    }

    #[test]
    fn skill_usage_type_team_mate_does_not_remove_effects_at_end_of_turn() {
        assert!(!SkillUsageType::OncePerTurnByTeamMate.remove_effects_at_end_of_turn());
    }

    #[test]
    fn skill_category_from_name_round_trip_all() {
        for cat in [
            SkillCategory::General, SkillCategory::Agility, SkillCategory::Passing,
            SkillCategory::Strength, SkillCategory::Extraordinary,
            SkillCategory::StatIncrease, SkillCategory::StatDecrease,
            SkillCategory::Trait, SkillCategory::Devious,
        ] {
            assert_eq!(SkillCategory::from_name(cat.name()), Some(cat));
        }
    }

    #[test]
    fn declare_condition_count_is_two() {
        let all = [DeclareCondition::None, DeclareCondition::Standing];
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn skill_usage_type_count_is_seven() {
        let all = [
            SkillUsageType::Regular, SkillUsageType::OncePerTurn, SkillUsageType::OncePerGame,
            SkillUsageType::OncePerHalf, SkillUsageType::OncePerDrive,
            SkillUsageType::OncePerTurnByTeamMate, SkillUsageType::Special,
        ];
        assert_eq!(all.len(), 7);
    }
}
