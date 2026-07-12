use crate::skill_behaviour::SkillBehaviour;

/// Dodge (BB2016): handles block-dodge pushback decisions and fall-through.
/// Mirrors Java `com.fumbbl.ffb.server.skillbehaviour.bb2016.DodgeBehaviour`.
///
/// Unlike the BB2020/BB2025 `AbstractDodgingBehaviour`-based translation (a real
/// `StepModifierTrait` dispatched via `SkillRegistry`/`execute_step_hooks`, see
/// `skill_behaviour/mixed/abstract_dodging_behaviour.rs`), BB2016's hand-rolled Java
/// `DodgeBehaviour` (its own inline anonymous `StepModifier`, not the shared abstract
/// base class) is translated directly into its target step:
/// `step/bb2016/block/step_block_dodge.rs` — that file's `find_dodge_choice` +
/// `execute_step` implement the real `findDodgeChoice`/`handleExecuteStepHook` logic
/// (dodge-choice heuristic, headless-resolved skill use, `ReportSkillUse`, restore-or-fall
/// state transition) with full unit test coverage. This type is therefore an intentionally
/// inert marker — not registered in `registry.rs`, since registering it would double-process
/// what the step already does directly.
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for DodgeBehaviour {
    fn name(&self) -> &'static str { "DodgeBehaviour" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_correct_string() {
        let b = DodgeBehaviour::new();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = DodgeBehaviour::default();
        assert_eq!(b.name(), "DodgeBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = DodgeBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2016,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = DodgeBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!DodgeBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = DodgeBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
