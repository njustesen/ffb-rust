use crate::skill_behaviour::SkillBehaviour;

/// BB2020 TheBallista skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.TheBallistaBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 always sets the re-rolled action to `THROW_TEAM_MATE`:
/// ```java
/// step.setReRolledAction(ReRolledActions.THROW_TEAM_MATE);
/// ```
///
/// BB2025 selects between `KICK_TEAM_MATE` and `THROW_TEAM_MATE` based on `state.kicked`:
/// ```java
/// ReRolledAction action = state.kicked ? ReRolledActions.KICK_TEAM_MATE : ReRolledActions.THROW_TEAM_MATE;
/// step.setReRolledAction(action);
/// ```
///
/// This means BB2020 does not support the kick-team-mate (KTM) re-roll distinction that BB2025
/// added for the Treeman/Halfling KTM mechanic.
pub struct TheBallistaBehaviour;

impl TheBallistaBehaviour {
    pub fn new() -> Self { Self }

    /// Returns the re-rolled action kind for TheBallista in **BB2020** (always ThrowTeamMate).
    ///
    /// In BB2025 this depends on whether the action is a kick (`state.kicked`).
    pub fn rerolled_action_bb2020(_kicked: bool) -> RerolledActionKind {
        // BB2020: always ThrowTeamMate, never KickTeamMate.
        RerolledActionKind::ThrowTeamMate
    }
}

/// Re-rolled action kinds relevant to TheBallista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerolledActionKind {
    ThrowTeamMate,
    KickTeamMate,
}

impl Default for TheBallistaBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for TheBallistaBehaviour {
    fn name(&self) -> &'static str { "TheBallistaBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: rerolled action is always ThrowTeamMate, even when kicked=true.
    #[test]
    fn bb2020_always_uses_throw_team_mate_when_kicked() {
        assert_eq!(
            TheBallistaBehaviour::rerolled_action_bb2020(true),
            RerolledActionKind::ThrowTeamMate
        );
    }

    /// BB2020: rerolled action is ThrowTeamMate when kicked=false.
    #[test]
    fn bb2020_always_uses_throw_team_mate_when_not_kicked() {
        assert_eq!(
            TheBallistaBehaviour::rerolled_action_bb2020(false),
            RerolledActionKind::ThrowTeamMate
        );
    }

    /// BB2020 never returns KickTeamMate.
    #[test]
    fn bb2020_never_uses_kick_team_mate() {
        for kicked in [true, false] {
            assert_ne!(
                TheBallistaBehaviour::rerolled_action_bb2020(kicked),
                RerolledActionKind::KickTeamMate,
                "BB2020 must never select KickTeamMate re-roll action"
            );
        }
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(TheBallistaBehaviour::new().name(), "TheBallistaBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = TheBallistaBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = TheBallistaBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
