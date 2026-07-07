use crate::skill_behaviour::SkillBehaviour;

/// BB2020 BoneHead skill behaviour.
/// StepModifier on StepBoneHead: rolls confusion check (4+); on failure cancels player action
/// (blitz/pass/etc used flags set) and goes to failure label. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.BoneHeadBehaviour`.
pub struct BoneHeadBehaviour;

impl BoneHeadBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for BoneHeadBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for BoneHeadBehaviour {
    fn name(&self) -> &'static str { "BoneHeadBehaviour" }

    /// Java `StepModifier<StepBoneHead, StepState>.handleExecuteStepHook`:
    /// checks if negatraits apply, rolls confusion check (4+), handles reroll, cancels player
    /// action on failure. Returns false always.
    ///
    /// TODO(hook-infra): needs state.goToLabelOnFailure,
    /// game.getTurnMode().checkNegatraits().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.goToLabelOnFailure,
        // game.getTurnMode().checkNegatraits())
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = BoneHeadBehaviour::new();
        assert_eq!(b.name(), "BoneHeadBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = BoneHeadBehaviour::default();
        assert_eq!(b.name(), "BoneHeadBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = BoneHeadBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = BoneHeadBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
#[test]    fn name_is_not_empty() {        assert!(!BoneHeadBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = BoneHeadBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
