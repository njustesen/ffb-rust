use crate::skill_behaviour::SkillBehaviour;

/// Stand Firm: may refuse a push result. Priority 1.
/// Auto-true if rooted; auto-false if prone/stunned or old state was prone.
/// If Juggernaut cancels during Blitz: auto-false with report. Otherwise asks defender.
/// On use: clears pushback stack, publishes FOLLOWUP_CHOICE=false, STARTING_PUSHBACK_SQUARE=null.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.StandFirmBehaviour`.
pub struct StandFirmBehaviour;

impl StandFirmBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StandFirmBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StandFirmBehaviour {
    fn name(&self) -> &'static str { "StandFirmBehaviour" }

    /// Java logic (handleExecuteStepHook, priority 1):
    ///   1. If player is Rooted: auto-use Stand Firm (return true, consume push).
    ///   2. If player is prone or stunned, or StepState.oldPlayerState was prone: auto-skip
    ///      (return false).
    ///   3. If Juggernaut cancelled Stand Firm during a Blitz action: publish STAND_FIRM_CANCELLED
    ///      report; return false.
    ///   4. Otherwise: show dialog asking defender to use Stand Firm.
    ///   5. On use: clear pushback stack (StepState.pushbackSquareStack),
    ///      publish FOLLOWUP_CHOICE report = false,
    ///      publish STARTING_PUSHBACK_SQUARE report = null.
    ///   6. Reads/writes: StepState.pushbackSquareStack, StepState.oldPlayerState,
    ///      StepState.juggernautCancelled.
    ///
    // TODO(hook-infra): step-specific state (StepState.pushbackSquareStack)
    // TODO(hook-infra): step-specific state (StepState.oldPlayerState)
    // TODO(hook-infra): step-specific state (StepState.juggernautCancelled)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = StandFirmBehaviour::new();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = StandFirmBehaviour::default();
        assert_eq!(b.name(), "StandFirmBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = StandFirmBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StandFirmBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
