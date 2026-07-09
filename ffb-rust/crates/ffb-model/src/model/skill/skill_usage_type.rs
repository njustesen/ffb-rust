/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillUsageType.
///
/// NOTE: The canonical enum lives in `crate::enums::SkillUsageType` (enums/skill.rs).
/// This file re-exports it for code that imports from `model::skill::skill_usage_type`.
pub use crate::enums::SkillUsageType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_does_not_track_outside_activation() {
        assert!(!SkillUsageType::Regular.track_outside_activation());
    }

    #[test]
    fn once_per_turn_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerTurn.track_outside_activation());
    }

    #[test]
    fn once_per_game_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerGame.track_outside_activation());
    }

    #[test]
    fn once_per_half_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerHalf.track_outside_activation());
    }

    #[test]
    fn once_per_drive_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerDrive.track_outside_activation());
    }

    #[test]
    fn once_per_turn_by_team_mate_tracks_outside_activation() {
        assert!(SkillUsageType::OncePerTurnByTeamMate.track_outside_activation());
    }

    #[test]
    fn special_tracks_outside_activation() {
        assert!(SkillUsageType::Special.track_outside_activation());
    }

    #[test]
    fn regular_removes_effects_at_end_of_turn() {
        assert!(SkillUsageType::Regular.remove_effects_at_end_of_turn());
    }

    #[test]
    fn special_does_not_remove_effects_at_end_of_turn() {
        assert!(!SkillUsageType::Special.remove_effects_at_end_of_turn());
    }

    #[test]
    fn once_per_turn_by_team_mate_does_not_remove_effects_at_end_of_turn() {
        assert!(!SkillUsageType::OncePerTurnByTeamMate.remove_effects_at_end_of_turn());
    }

    #[test]
    fn once_per_turn_removes_effects_at_end_of_turn() {
        assert!(SkillUsageType::OncePerTurn.remove_effects_at_end_of_turn());
    }
}
