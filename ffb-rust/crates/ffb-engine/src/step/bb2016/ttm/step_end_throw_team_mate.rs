/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2016.ttm.StepEndThrowTeamMate`.
///
/// Final step of the throw-team-mate sequence. Resets thrown player position/state
/// to their pre-throw values, clears pass coordinate and range ruler, then pushes
/// the EndPlayerAction generator sequence.
///
/// Consumes: END_TURN, THROWN_PLAYER_COORDINATE, THROWN_PLAYER_HAS_BALL,
///           THROWN_PLAYER_ID, THROWN_PLAYER_STATE.
///
/// client-only: UtilServerDialog.hideDialog (dialog layer not translated).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::PlayerState;
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepAction, StepId, StepParameter};
use crate::step::generator::bb2016::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2016::Select;
use crate::step::generator::bb2016::select::SelectParams;

/// Java: `StepEndThrowTeamMate` (bb2016/ttm).
pub struct StepEndThrowTeamMate {
    /// Java: `fEndTurn`
    end_turn: bool,
    /// Java: `fEndPlayerAction`
    end_player_action: bool,
    /// Java: `fThrownPlayerCoordinate`
    thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: bool,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fThrownPlayerState`
    thrown_player_state: Option<PlayerState>,
}

impl StepEndThrowTeamMate {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            thrown_player_coordinate: None,
            thrown_player_has_ball: false,
            thrown_player_id: None,
            thrown_player_state: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // client-only: UtilServerDialog.hideDialog
        game.pass_coordinate = None;
        game.field_model.range_ruler = None;
        game.defender_id = None;

        // Reset thrown player to pre-throw coordinate/state if all values present.
        if let (Some(id), Some(coord), Some(state)) = (
            &self.thrown_player_id,
            self.thrown_player_coordinate,
            self.thrown_player_state,
        ) {
            // Only reset when state.id > 0 (i.e. a real player state, not cleared).
            if state.0 > 0 {
                game.field_model.set_player_coordinate(id, coord);
                game.field_model.set_player_state(id, state);
                if self.thrown_player_has_ball {
                    game.field_model.ball_coordinate = Some(coord);
                }
            }
        }
        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
        });
        StepOutcome::next().push_seq(seq)
    }
}

impl Default for StepEndThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndThrowTeamMate {
    fn id(&self) -> StepId { StepId::EndThrowTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_ACTING_PLAYER → Select.pushSequence(false) + NEXT_STEP_AND_REPEAT.
            Action::ActivatePlayer { .. } => {
                let select_seq = Select::build_sequence(&SelectParams { update_persistence: false });
                return StepOutcome {
                    action: StepAction::NextStepAndRepeat,
                    goto_label: None,
                    published: vec![],
                    pushes: vec![select_seq],
                    events: vec![],
                    prompt: None,
                    clear_stack: false,
                };
            }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)                  => { self.end_turn = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)   => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrownPlayerHasBall(v)      => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerId(v)           => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)        => { self.thrown_player_state = Some(*v); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn id_is_end_throw_team_mate() {
        assert_eq!(StepEndThrowTeamMate::new().id(), StepId::EndThrowTeamMate);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let out = StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn clears_pass_coordinate() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate { x: 5, y: 5 });
        StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn clears_range_ruler() {
        let mut game = make_game();
        game.field_model.range_ruler = Some(ffb_model::types::RangeRuler::new("p".into(), None, 3, true));
        StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_none());
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn start_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let out = StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty());
    }

    #[test]
    fn activate_player_pushes_select_and_next_step_and_repeat() {
        use crate::action::PlayerActionChoice;
        let mut game = make_game();
        let mut step = StepEndThrowTeamMate::new();
        let out = step.handle_command(
            &Action::ActivatePlayer { player_id: "p1".into(), player_action: PlayerActionChoice::Move, block_defender_id: None },
            &mut game,
            &mut GameRng::new(0),
        );
        // Java: Select.pushSequence(false) + NEXT_STEP_AND_REPEAT
        assert_eq!(out.action, StepAction::NextStepAndRepeat);
        assert_eq!(out.pushes.len(), 1);
    }
}
