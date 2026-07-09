use crate::skill_behaviour::SkillBehaviour;

/// BB2020 MonstrousMouth skill behaviour.
///
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2020.MonstrousMouthBehaviour`.
///
/// **BB2020 vs BB2025 difference:**
///
/// BB2020 registers one modifier on `StepCatchScatterThrowIn`: when the catcher has MonstrousMouth
/// the skill enables a catch re-roll (sets `rerolledAction = CATCH` and `rerollCatch = true`).
///
/// BB2025 completely replaces this with a modifier on `StepPushback` (priority 1): when the
/// defender is in the "chomped" player-state it forces the pushback to proceed (clears
/// `pushbackStack`, sets `doPush = true`) and suppresses strip-ball, emitting a report event if
/// the chomped player had the ball.
///
/// The two editions therefore have **entirely different step targets and semantics**:
/// - BB2020: catch-phase re-roll mechanic.
/// - BB2025: pushback-phase chomped-state override (strip-ball prevention).
pub struct MonstrousMouthBehaviour;

impl MonstrousMouthBehaviour {
    pub fn new() -> Self { Self }

    /// Returns `true` when this edition's MonstrousMouth modifier applies to the catch phase
    /// (BB2020), or `false` when it applies to the pushback phase (BB2025).
    pub const fn applies_in_catch_phase() -> bool {
        true
    }

    /// Returns `true` when this edition's MonstrousMouth modifier applies to the pushback phase
    /// (BB2025), or `false` when it applies to the catch phase (BB2020).
    pub const fn applies_in_pushback_phase() -> bool {
        false
    }
}

impl Default for MonstrousMouthBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MonstrousMouthBehaviour {
    fn name(&self) -> &'static str { "MonstrousMouthBehaviour" }

    /// TODO(hook-infra): step-specific state access not yet wired.
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BB2020 MonstrousMouth applies in the catch phase.
    #[test]
    fn bb2020_applies_in_catch_phase() {
        assert!(MonstrousMouthBehaviour::applies_in_catch_phase());
    }

    /// BB2020 MonstrousMouth does NOT apply in the pushback phase.
    #[test]
    fn bb2020_does_not_apply_in_pushback_phase() {
        assert!(!MonstrousMouthBehaviour::applies_in_pushback_phase());
    }

    /// The two constants are mutually exclusive.
    #[test]
    fn catch_and_pushback_phases_are_mutually_exclusive() {
        assert_ne!(
            MonstrousMouthBehaviour::applies_in_catch_phase(),
            MonstrousMouthBehaviour::applies_in_pushback_phase()
        );
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(MonstrousMouthBehaviour::new().name(), "MonstrousMouthBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = MonstrousMouthBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = MonstrousMouthBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, before);
    }
}
