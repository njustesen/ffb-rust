use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::EndPlayerAction;
use crate::step::generator::bb2016::end_player_action::EndPlayerActionParams;

/// Final step of the foul sequence (BB2016).
/// Always pushes EndPlayerAction with feedingAllowed=true, endPlayerAction=true.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.foul.StepEndFouling.
pub struct StepEndFouling {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction (serialized only, not used in executeStep)
    pub end_player_action: bool,
}

impl StepEndFouling {
    pub fn new() -> Self {
        Self { end_turn: false, end_player_action: false }
    }
}

impl Default for StepEndFouling {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndFouling {
    fn id(&self) -> StepId { StepId::EndFouling }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

impl StepEndFouling {
    fn execute_step(&self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: EndPlayerAction.pushSequence(new SequenceParams(getGameState(), true, true, fEndTurn))
        //   feedingAllowed=true, endPlayerAction=true, endTurn=fEndTurn
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
        });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepId};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndFouling::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::InitFeeding);
    }

    #[test]
    fn end_turn_passes_through_to_sequence() {
        let mut game = make_game();
        let mut step = StepEndFouling::new();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Verify init_feeding step has EndTurn=true param
        let init_feeding_params = &out.pushes[0][0].params;
        assert!(init_feeding_params.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndFouling::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepEndFouling::new();
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepEndFouling::new();
        assert!(!step.set_parameter(&StepParameter::HomeTeam(true)));
    }
}
