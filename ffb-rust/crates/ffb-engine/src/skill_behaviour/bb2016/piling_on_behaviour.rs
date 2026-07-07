use crate::skill_behaviour::SkillBehaviour;

/// Piling On: attacker may drop prone on the defender to re-roll armour or injury.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.PilingOnBehaviour`.
pub struct PilingOnBehaviour;

impl PilingOnBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PilingOnBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PilingOnBehaviour {
    fn name(&self) -> &'static str { "PilingOnBehaviour" }

    /// Java `StepModifier.handleExecuteStepHook` logic (StepBlockInjury context) — two phases:
    ///
    /// **Phase 1 (initial pass, `StepState.usingPilingOn` not yet set):**
    /// 1. Drop the defender (apply prone state).
    /// 2. Roll the initial injury result for the defender.
    /// 3. If the Piling On skill is available for the attacker:
    ///    show a dialog asking whether to use Piling On.
    ///    - On decline: publish result as-is and finish.
    ///    - On confirm: set `StepState.usingPilingOn = true` and loop back.
    ///
    /// **Phase 2 (`StepState.usingPilingOn == true`):**
    /// 1. Drop the attacker prone (they fall on the defender).
    /// 2. Re-roll the injury (armour roll or injury roll as configured by the dialog choice).
    /// 3. Publish the new (potentially worse) result.
    ///
    /// **Attacker-falling path:**
    /// If the attacker themselves is already falling, suppress Piling On entirely.
    ///
    /// All step-local state fields are unavailable in the current Rust signature:
    // TODO(hook-infra): step-specific state (StepState.usingPilingOn)
    // TODO(hook-infra): dialog fields (armour-or-injury choice, confirm/decline)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = PilingOnBehaviour::new();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = PilingOnBehaviour::default();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = PilingOnBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PilingOnBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
