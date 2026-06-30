use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::check_touchdown;
use crate::step::generator::bb2025::EndPlayerAction;
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;

/// Final step of the punt sequence. Consumes CatcherId/PlayerId/EndTurn, then pushes the
/// EndPlayerAction sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.punt.StepEndPunt`.
pub struct StepEndPunt {
    pub catcher_id: Option<String>,
    pub ball_snatcher_id: Option<String>,
    pub end_turn: bool,
}

impl StepEndPunt {
    pub fn new() -> Self {
        Self { catcher_id: None, ball_snatcher_id: None, end_turn: false }
    }
}

impl Default for StepEndPunt {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndPunt {
    fn id(&self) -> StepId { StepId::EndPunt }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            StepParameter::PlayerId(v) => { self.ball_snatcher_id = Some(v.clone()); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepEndPunt {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO: UtilServerDialog.hideDialog(gameState)
        // Java: endTurn |= UtilServerSteps.checkTouchdown(gameState)
        self.end_turn |= check_touchdown(game);

        // Java: endGenerator.pushSequence(new EndPlayerAction.SequenceParams(gameState, true, true, endTurn))
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
            check_forgo: false,
        });
        StepOutcome::next().push_seq(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    // 1. Default → NextStep with EndPlayerAction sequence pushed
    #[test]
    fn default_start_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndPunt::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    // 2. set_parameter CatcherId is consumed
    #[test]
    fn set_parameter_catcher_id() {
        let mut step = StepEndPunt::new();
        assert!(step.catcher_id.is_none());
        let consumed = step.set_parameter(&StepParameter::CatcherId(Some("catcher1".into())));
        assert!(consumed);
        assert_eq!(step.catcher_id.as_deref(), Some("catcher1"));
    }

    // 3. set_parameter EndTurn is consumed
    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepEndPunt::new();
        assert!(!step.end_turn);
        let consumed = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(consumed);
        assert!(step.end_turn);
    }

    // 4. set_parameter PlayerId (ball_snatcher_id) is consumed
    #[test]
    fn set_parameter_player_id() {
        let mut step = StepEndPunt::new();
        assert!(step.ball_snatcher_id.is_none());
        let consumed = step.set_parameter(&StepParameter::PlayerId("snatcher1".into()));
        assert!(consumed);
        assert_eq!(step.ball_snatcher_id.as_deref(), Some("snatcher1"));
    }

    // 5. handle_command also returns NextStep
    #[test]
    fn handle_command_returns_next_step() {
        use crate::action::Action;
        let mut game = make_game();
        let mut step = StepEndPunt::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
