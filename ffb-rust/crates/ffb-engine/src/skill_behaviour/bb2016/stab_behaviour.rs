use crate::skill_behaviour::SkillBehaviour;

/// Stab: may stab an adjacent opponent instead of blocking.
/// If usingStab: plays STAB sound, handles injury (InjuryTypeStab), drops player if armour
/// broken, publishes INJURY_RESULT and GOTO_LABEL goToLabelOnSuccess.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.StabBehaviour`.
pub struct StabBehaviour;

impl StabBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StabBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StabBehaviour {
    fn name(&self) -> &'static str { "StabBehaviour" }

    /// Java logic (handleExecuteStepHook):
    ///   1. Check StepState.usingStab; if false, skip (no-op).
    ///   2. Play STAB sound effect.
    ///   3. Roll armour using InjuryTypeStab mechanics.
    ///   4. If armour broken: drop (prone) the target player; publish INJURY_RESULT report.
    ///   5. Push GOTO_LABEL command with StepState.goToLabelOnSuccess as target.
    ///   6. Reads/writes: StepState.usingStab, StepState.goToLabelOnSuccess.
    ///
    // TODO(hook-infra): step-specific state (StepState.usingStab)
    // TODO(hook-infra): step-specific state (StepState.goToLabelOnSuccess)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = StabBehaviour::new();
        assert_eq!(b.name(), "StabBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = StabBehaviour::default();
        assert_eq!(b.name(), "StabBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = StabBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = StabBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
