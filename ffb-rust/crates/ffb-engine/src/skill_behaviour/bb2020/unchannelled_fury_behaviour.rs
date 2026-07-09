use crate::skill_behaviour::SkillBehaviour;

/// BB2020 UnchannelledFury skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.UnchannelledFuryBehaviour`.
///
/// **BB2020 vs BB2025 difference (cancelPlayerAction helper):**
///
/// Identical pattern to ReallyStupid / BoneHead: BB2020 combines `PASS`/`PASS_MOVE` and
/// `THROW_TEAM_MATE`/`THROW_TEAM_MATE_MOVE` under `setPassUsed(true)`. BB2025 separates them
/// and adds `PUNT`/`PUNT_MOVE` → `setPuntUsed(true)`.
///
/// All confusion-roll mechanics (roll formula, re-roll, dialog) are identical between editions.
pub struct UnchannelledFuryBehaviour;

impl UnchannelledFuryBehaviour {
    pub fn new() -> Self { Self }

    /// Classify the player action into the BB2020 turn-data flag to mark used when
    /// UnchannelledFury causes the action to be cancelled.
    pub fn turn_data_flag_for_action_bb2020(action: UfActionKind) -> UfTurnDataFlag {
        match action {
            UfActionKind::Blitz
            | UfActionKind::BlitzMove
            | UfActionKind::KickEmBlitz => UfTurnDataFlag::BlitzUsed,

            UfActionKind::KickTeamMate
            | UfActionKind::KickTeamMateMove => UfTurnDataFlag::KtmUsed,

            // BB2020: ThrowTeamMate shares passUsed with Pass.
            UfActionKind::Pass
            | UfActionKind::PassMove
            | UfActionKind::ThrowTeamMate
            | UfActionKind::ThrowTeamMateMove => UfTurnDataFlag::PassUsed,

            UfActionKind::HandOver
            | UfActionKind::HandOverMove => UfTurnDataFlag::HandOverUsed,

            UfActionKind::Foul
            | UfActionKind::FoulMove => UfTurnDataFlag::FoulUsed,

            // BB2020: no PUNT, no SECURE_THE_BALL.
            _ => UfTurnDataFlag::None,
        }
    }
}

/// Player action kinds relevant to UnchannelledFury cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfActionKind {
    Blitz,
    BlitzMove,
    KickEmBlitz,
    KickTeamMate,
    KickTeamMateMove,
    Pass,
    PassMove,
    ThrowTeamMate,
    ThrowTeamMateMove,
    HandOver,
    HandOverMove,
    Foul,
    FoulMove,
    Punt,
    PuntMove,
    Other,
}

/// Turn-data flags for UnchannelledFury.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfTurnDataFlag {
    BlitzUsed,
    KtmUsed,
    PassUsed,
    HandOverUsed,
    FoulUsed,
    PuntUsed,
    None,
}

impl Default for UnchannelledFuryBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for UnchannelledFuryBehaviour {
    fn name(&self) -> &'static str { "UnchannelledFuryBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020: ThrowTeamMate maps to PassUsed.
    #[test]
    fn throw_team_mate_uses_pass_flag_bb2020() {
        assert_eq!(
            UnchannelledFuryBehaviour::turn_data_flag_for_action_bb2020(UfActionKind::ThrowTeamMate),
            UfTurnDataFlag::PassUsed
        );
    }

    /// BB2020: Punt has no turn-data flag (Punt is a BB2025-only action).
    #[test]
    fn punt_has_no_flag_in_bb2020() {
        assert_eq!(
            UnchannelledFuryBehaviour::turn_data_flag_for_action_bb2020(UfActionKind::Punt),
            UfTurnDataFlag::None
        );
    }

    /// BB2020: Blitz uses BlitzUsed flag.
    #[test]
    fn blitz_uses_blitz_flag_bb2020() {
        assert_eq!(
            UnchannelledFuryBehaviour::turn_data_flag_for_action_bb2020(UfActionKind::Blitz),
            UfTurnDataFlag::BlitzUsed
        );
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(UnchannelledFuryBehaviour::new().name(), "UnchannelledFuryBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = UnchannelledFuryBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = UnchannelledFuryBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
