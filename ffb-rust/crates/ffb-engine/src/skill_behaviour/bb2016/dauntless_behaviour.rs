use crate::skill_behaviour::SkillBehaviour;

/// Dauntless: attacker may match or exceed defender Strength when outnumbered.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.DauntlessBehaviour`.
pub struct DauntlessBehaviour;

impl DauntlessBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DauntlessBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DauntlessBehaviour {
    fn name(&self) -> &'static str { "DauntlessBehaviour" }

    /// Java `StepDauntless.handleExecuteStepHook` logic (condensed):
    ///
    /// Pre-condition: only fires when `attacker.getStrength() < defender.getStrength()`
    ///                AND the active action is not STAB (stab bypasses dauntless).
    ///
    /// 1. Roll a D6; the target is `defender.getStrength() - attacker.getStrength() + 1`
    ///    (i.e. roll at least that value to succeed — equivalent to Java
    ///    `minimumRollDauntless()`).
    /// 2. On success:
    ///    → publish `SUCCESSFUL_DAUNTLESS` (the attacker's effective ST is raised to
    ///      match the defender's ST for the block).
    ///    → `setNextAction(NEXT_STEP)`.
    /// 3. On failure:
    ///    a. If a reroll is available (DAUNTLESS reroll source and not already rerolled):
    ///       → ask for reroll dialog; on retry go back to step 1.
    ///    b. If no reroll → `setNextAction(NEXT_STEP)` (block proceeds at normal ST).
    ///
    /// TODO(hook-infra): step-specific state (StepState dauntless-roll fields,
    ///                   step.reRolledAction, step.reRollSource).
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DauntlessBehaviour::new();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DauntlessBehaviour::default();
        assert_eq!(b.name(), "DauntlessBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DauntlessBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DauntlessBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
