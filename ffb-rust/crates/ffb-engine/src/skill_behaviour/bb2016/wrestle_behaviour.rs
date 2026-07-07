use crate::skill_behaviour::SkillBehaviour;

/// Wrestle: player may choose to knock both players down after a block.
/// Three-phase dialog: asks attacker then defender. Either party can use Wrestle.
/// On use: drops both players. If defender has placedProneCausesInjuryRoll (Ball & Chain),
/// handles injury. Juggernaut cancels defender use during blitz.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.WrestleBehaviour`.
pub struct WrestleBehaviour;

impl WrestleBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for WrestleBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for WrestleBehaviour {
    fn name(&self) -> &'static str { "WrestleBehaviour" }

    /// Java logic (handleExecuteStepHook — three-phase dialog):
    ///
    /// Phase 1 — ask attacker:
    ///   1. If attacker has Wrestle skill: show dialog asking if they want to use it.
    ///   2. If attacker chooses yes: set StepState.usingWrestle = ATTACKER; proceed to phase 3.
    ///   3. If attacker chooses no: proceed to phase 2.
    ///
    /// Phase 2 — ask defender:
    ///   1. If defender has Wrestle skill: show dialog asking if they want to use it.
    ///   2. If Juggernaut is active (blitz action): suppress defender Wrestle; auto-skip.
    ///   3. If defender chooses yes: set StepState.usingWrestle = DEFENDER; proceed to phase 3.
    ///
    /// Phase 3 — apply Wrestle:
    ///   1. Drop both attacker and defender to PRONE.
    ///   2. If defender has placedProneCausesInjuryRoll (Ball & Chain player):
    ///      handle injury roll for the Ball & Chain player.
    ///   3. Publish WRESTLE_USED report.
    ///   4. Reads/writes: StepState.usingWrestle, StepState.juggernautActive,
    ///      StepState.placedProneCausesInjuryRoll.
    ///
    // TODO(hook-infra): step-specific state (StepState.usingWrestle)
    // TODO(hook-infra): step-specific state (StepState.juggernautActive)
    // TODO(hook-infra): step-specific state (StepState.placedProneCausesInjuryRoll)
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = WrestleBehaviour::new();
        assert_eq!(b.name(), "WrestleBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = WrestleBehaviour::default();
        assert_eq!(b.name(), "WrestleBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = WrestleBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = WrestleBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
