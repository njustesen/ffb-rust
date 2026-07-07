use crate::skill_behaviour::SkillBehaviour;

/// BB2020 ReallyStupid skill behaviour. StepModifier on StepReallyStupid: rolls confusion check
/// (4+) each activation; on failure marks player confused and goes to failure label. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2020.ReallyStupidBehaviour`.
pub struct ReallyStupidBehaviour;

impl ReallyStupidBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ReallyStupidBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ReallyStupidBehaviour {
    fn name(&self) -> &'static str { "ReallyStupidBehaviour" }

    /// Java `StepModifier<StepReallyStupid, StepState>.handleExecuteStepHook`: rolls confusion
    /// check (4+) each activation; on failure marks player confused and goes to failure label.
    /// Returns false always.
    /// TODO(hook-infra): needs state.goToLabelOnFailure, game.getTurnMode().checkNegatraits().
    fn execute_step_hook(&self, _game: &mut ffb_model::model::game::Game) -> bool {
        // TODO(hook-infra): step-specific state access (StepState.xxx)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_is_noop_returns_false() {
        // Without step infra the hook always returns false.
        let b = ReallyStupidBehaviour::new();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }

    #[test]
    fn name_is_correct() {
        let b = ReallyStupidBehaviour::default();
        assert_eq!(b.name(), "ReallyStupidBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let b = ReallyStupidBehaviour::new();
        let mut game = ffb_model::model::game::Game::new(
            test_team("home", 0), test_team("away", 0), Rules::Bb2020,
        );
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = ReallyStupidBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }
}
