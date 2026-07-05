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
/// DEFERRED(EndTTM-dialog): UtilServerDialog.hideDialog deferred (dialog-client).
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PlayerAction};
use ffb_model::types::FieldCoordinate;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepAction, StepId, StepParameter};
use crate::step::generator::bb2020::end_player_action::{EndPlayerAction, EndPlayerActionParams};
use crate::step::generator::bb2020::move_::{Move as MoveGenerator, MoveParams};
use crate::step::generator::bb2020::Select;
use crate::step::generator::bb2020::select::SelectParams;

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
        // DEFERRED(EndTTM-dialog): UtilServerDialog.hideDialog(gameState).
        game.pass_coordinate = None;
        game.field_model.range_ruler = None;
        game.defender_id = None;
        game.thrower_id = None;

        // Java: actingPlayer.isSufferingBloodLust() && bloodlustAction != null
        let move_due_to_bloodlust = game.acting_player.suffering_blood_lust
            && self.bloodlust_action.is_some();

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
            // Java: UtilServerGame.syncGameModel (clear acting player move data)
            // Java: UtilServerSteps.changePlayerAction(actingPlayer, bloodlustAction)
            game.acting_player.player_action = self.bloodlust_action;
            let seq = MoveGenerator::build_sequence(&MoveParams {
                bloodlust_action: self.bloodlust_action,
                ..Default::default()
            });
            StepOutcome::next().push_seq(seq)
        } else {
            // Java: EndPlayerAction.pushSequence(feedingAllowed=true, endPlayerAction=true, endTurn)
            let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
                feeding_allowed: true,
                end_player_action: true,
                end_turn: self.end_turn,
            });
            StepOutcome::next().push_seq(seq)
        }
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
        if let Action::ActivatePlayer { .. } = action {
            // Java: CLIENT_ACTING_PLAYER → Select.pushSequence(false) + NEXT_STEP_AND_REPEAT.
            let select_seq = Select::build_sequence(&SelectParams {
                update_persistence: false,
                is_blitz_move: false,
                block_targets: vec![],
            });
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
            StepParameter::BloodLustAction(v)         => { self.bloodlust_action = *v; true }
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

    #[test]
    fn start_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let out = StepEndThrowTeamMate::new().start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty(), "should push EndPlayerAction sequence");
    }

    #[test]
    fn bloodlust_active_pushes_move_sequence() {
        let mut game = make_game();
        game.acting_player.suffering_blood_lust = true;
        let mut step = StepEndThrowTeamMate::new();
        step.bloodlust_action = Some(PlayerAction::Move);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.pushes.is_empty(), "should push Move sequence for bloodlust");
    }

    #[test]
    fn set_parameter_blood_lust_action() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::BloodLustAction(Some(PlayerAction::Move))));
        assert_eq!(step.bloodlust_action, Some(PlayerAction::Move));
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
