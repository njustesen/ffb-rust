/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepEndThrowTeamMate`.
///
/// Final step of the throw-team-mate sequence. Resets thrown player position/state
/// to their pre-throw values, clears pass coordinate, range ruler, defender and thrower IDs,
/// then pushes the EndPlayerAction generator sequence (or Move sequence when bloodlust active).
///
/// BB2020 differences vs BB2016:
///  - Also clears `game.thrower_id`.
///  - Adds `old_player_state` (OLD_DEFENDER_STATE) for reset logic.
///  - Adds `bloodlust_action` (BLOOD_LUST_ACTION) for bloodlust Move redirect.
///  - On CLIENT_ACTING_PLAYER command pushes a Select sequence then NEXT_STEP_AND_REPEAT.
///
/// Consumes: END_TURN, END_PLAYER_ACTION, THROWN_PLAYER_COORDINATE, THROWN_PLAYER_HAS_BALL,
///           THROWN_PLAYER_ID, THROWN_PLAYER_STATE, OLD_DEFENDER_STATE, BLOOD_LUST_ACTION.
///
/// TODO(EndTTM-generator): EndPlayerAction / Move SequenceGenerator not yet ported for BB2020.
/// TODO(EndTTM-dialog): UtilServerDialog.hideDialog deferred.
/// TODO(EndTTM-bloodlust): UtilServerGame.syncGameModel / UtilServerSteps.changePlayerAction deferred.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PlayerAction};
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepEndThrowTeamMate` (bb2020/ttm).
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
    /// Java: `oldPlayerState` (OLD_DEFENDER_STATE) — BB2020 addition.
    old_player_state: Option<PlayerState>,
    /// Java: `bloodlustAction` (BLOOD_LUST_ACTION) — BB2020 addition.
    bloodlust_action: Option<PlayerAction>,
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
            old_player_state: None,
            bloodlust_action: None,
        }
    }

    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // TODO(EndTTM-dialog): UtilServerDialog.hideDialog(gameState).
        game.pass_coordinate = None;
        game.field_model.range_ruler = None;
        game.defender_id = None;
        game.thrower_id = None;

        // TODO(EndTTM-bloodlust): BB2020 bloodlust check: actingPlayer.isSufferingBloodLust() && bloodlustAction != null.
        let move_due_to_bloodlust = false; // stub — bloodlust detection deferred

        // Reset thrown player to pre-throw coordinate/state if all values present.
        if let (Some(id), Some(coord)) = (
            &self.thrown_player_id,
            self.thrown_player_coordinate,
        ) {
            if game.player(id).is_some() {
                // BB2020: prefer oldPlayerState when endPlayerAction or bloodlust; else thrownPlayerState.
                if self.end_player_action || move_due_to_bloodlust {
                    if let Some(old) = self.old_player_state {
                        if old.id() > 0 {
                            game.field_model.set_player_state(id, old);
                        }
                    }
                } else if let Some(state) = self.thrown_player_state {
                    if state.id() > 0 {
                        game.field_model.set_player_state(id, state);
                    }
                }
                game.field_model.set_player_coordinate(id, coord);
                if self.thrown_player_has_ball {
                    game.field_model.ball_coordinate = Some(coord);
                }
            }
        }

        if move_due_to_bloodlust {
            // TODO(EndTTM-bloodlust): syncGameModel; changePlayerAction; push Move sequence.
        } else {
            // TODO(EndTTM-generator): EndPlayerAction.pushSequence(true, true, end_turn).
        }
        StepOutcome::next()
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

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // TODO(EndTTM-selectGenerator): CLIENT_ACTING_PLAYER → push Select sequence + NEXT_STEP_AND_REPEAT.
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)                 => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)         => { self.end_player_action = *v; true }
            StepParameter::ThrownPlayerCoordinate(v)  => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrownPlayerHasBall(v)     => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerId(v)          => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v)       => { self.thrown_player_state = Some(*v); true }
            StepParameter::OldDefenderState(v)        => { self.old_player_state = Some(*v); true }
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
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
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
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));
        StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn clears_range_ruler() {
        let mut game = make_game();
        game.field_model.range_ruler = None;
        StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.range_ruler.is_none());
    }

    #[test]
    fn clears_thrower_id() {
        let mut game = make_game();
        game.thrower_id = Some("thrower".into());
        StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(game.thrower_id.is_none());
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_old_defender_state() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut step = StepEndThrowTeamMate::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_player_state, Some(state));
    }
}
