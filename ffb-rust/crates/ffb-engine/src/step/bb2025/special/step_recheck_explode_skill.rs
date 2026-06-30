use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// Rechecks whether a bomb should explode on the Bombardier's own square.
///
/// This step corresponds to the second check of the Explode skill after a
/// Bombardier catches or has a bomb land on their own square. Java source
/// not found in the server repo — it is referenced only by StepId::RECHECK_EXPLODE_SKILL
/// and the step registry. Behaviour: check if the acting player still has an
/// unused canForceBombExplosion skill; if so, show the dialog again; else
/// proceed to the next step.
///
/// TODO: Explode skill re-check dialog not yet ported.
///   Java path: if actingPlayer has unused canForceBombExplosion skill -> show dialog
///   else -> NEXT_STEP
pub struct StepRecheckExplodeSkill;

impl StepRecheckExplodeSkill {
    pub fn new() -> Self { Self }
}

impl Default for StepRecheckExplodeSkill {
    fn default() -> Self { Self::new() }
}

impl Step for StepRecheckExplodeSkill {
    fn id(&self) -> StepId { StepId::RecheckExplodeSkill }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO: check actingPlayer.hasUnusedSkillWithProperty(canForceBombExplosion)
        //   if true -> showDialog(DialogSkillUseParameter) -> Continue
        //   else -> NEXT_STEP
        StepOutcome::next()
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO: CLIENT_USE_SKILL -> store decision -> EXECUTE_STEP -> check and proceed
        StepOutcome::next()
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepRecheckExplodeSkill::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_always_returns_false() {
        let mut step = StepRecheckExplodeSkill::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn default_and_new_are_equivalent() {
        let _a = StepRecheckExplodeSkill::new();
        let _b = StepRecheckExplodeSkill::default();
        // Both unit structs — just verify they compile and id() matches
    }
}
