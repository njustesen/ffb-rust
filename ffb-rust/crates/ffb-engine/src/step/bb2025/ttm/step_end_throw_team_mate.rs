use ffb_model::enums::{PlayerAction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter, SequenceStep};
use crate::step::generator::bb2025::{EndPlayerAction, Move, Select};
use crate::step::generator::bb2025::end_player_action::EndPlayerActionParams;
use crate::step::generator::bb2025::move_::MoveParams;
use crate::step::generator::bb2025::select::SelectParams;
use crate::step::util_server_steps;

/// Finalises the throw-team-mate sequence. Resets thrown player state/position,
/// then routes to Move (blood lust) or EndPlayerAction.
/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.ttm.StepEndThrowTeamMate.
pub struct StepEndThrowTeamMate {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: checkForgo
    pub check_forgo: bool,
    /// Java: fThrownPlayerCoordinate
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    /// Java: fThrownPlayerHasBall
    pub thrown_player_has_ball: bool,
    /// Java: fThrownPlayerId
    pub thrown_player_id: Option<String>,
    /// Java: fThrownPlayerState
    pub thrown_player_state: Option<PlayerState>,
    /// Java: oldPlayerState
    pub old_player_state: Option<PlayerState>,
    /// Java: bloodlustAction (PlayerAction)
    pub bloodlust_action: Option<PlayerAction>,
}

impl StepEndThrowTeamMate {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            check_forgo: false,
            thrown_player_coordinate: None,
            thrown_player_has_ball: false,
            thrown_player_id: None,
            thrown_player_state: None,
            old_player_state: None,
            bloodlust_action: None,
        }
    }
}

impl Default for StepEndThrowTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndThrowTeamMate {
    fn id(&self) -> StepId { StepId::EndThrowTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { ..
} => {
                // Java: CLIENT_ACTING_PLAYER → push Select sequence → NEXT_STEP_AND_REPEAT (SKIP_STEP)
                let seq = Select::build_sequence(&SelectParams { update_persistence: false, is_blitz_move: false, ..Default::default() });
                return StepOutcome {
                    action: StepAction::NextStepAndRepeat,
                    goto_label: None,
                    events: vec![],
                    pushes: vec![seq],
                    published: vec![],
                    prompt: None,
                    clear_stack: false,
                };
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::ThrownPlayerCoordinate(v) => { self.thrown_player_coordinate = *v; true }
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = *v; true }
            StepParameter::ThrownPlayerId(v) => { self.thrown_player_id = v.clone(); true }
            StepParameter::ThrownPlayerState(v) => { self.thrown_player_state = Some(*v); true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::OldDefenderState(v) => { self.old_player_state = Some(*v); true }
            StepParameter::BloodLustAction(v) => { self.bloodlust_action = *v; true }
            StepParameter::CheckForgo(v) => { self.check_forgo = *v; true }
            _ => false,
        }
    }
}

impl StepEndThrowTeamMate {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog — no-op in headless Rust
        game.pass_coordinate = None;
        game.field_model.range_ruler = None;
        game.defender_id = None;
        game.thrower_id = None;

        // Java: moveDueToBloodlust = actingPlayer.isSufferingBloodLust() && bloodlustAction != null
        let suffering_blood_lust = game.acting_player.suffering_blood_lust;
        let move_due_to_bloodlust = suffering_blood_lust && self.bloodlust_action.is_some();

        // Java: reset thrown player position/state
        if let (Some(ref thrown_id), Some(thrown_coord)) = (&self.thrown_player_id, self.thrown_player_coordinate) {
            // Restore state
            if self.end_player_action || move_due_to_bloodlust {
                if let Some(old_state) = self.old_player_state {
                    if old_state.0 > 0 {
                        game.field_model.set_player_state(thrown_id, old_state);
                    }
                }
            } else if let Some(thrown_state) = self.thrown_player_state {
                if thrown_state.0 > 0 {
                    game.field_model.set_player_state(thrown_id, thrown_state);
                }
            }
            game.field_model.set_player_coordinate(thrown_id, thrown_coord);
            if self.thrown_player_has_ball {
                game.field_model.ball_coordinate = Some(thrown_coord);
            }
        }

        if move_due_to_bloodlust {
            if let (Some(ref pid), Some(action)) = (&game.acting_player.player_id.clone(), self.bloodlust_action) {
                util_server_steps::change_player_action(game, pid, action, false);
            }
            let seq = Move::build_sequence(&MoveParams::default());
            return StepOutcome::next().push_seq(seq);
        }

        let seq = EndPlayerAction::build_sequence(&EndPlayerActionParams {
            feeding_allowed: true,
            end_player_action: true,
            end_turn: self.end_turn,
            check_forgo: self.check_forgo,
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
    use ffb_model::types::FieldCoordinate;
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn default_start_pushes_end_player_action_sequence() {
        let mut game = make_game();
        let mut step = StepEndThrowTeamMate::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(out.pushes.len(), 1);
        assert_eq!(out.pushes[0][0].step_id, StepId::RemoveTargetSelectionState);
    }

    #[test]
    fn start_clears_pass_coordinate_and_ids() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));
        game.defender_id = Some("d1".into());
        game.thrower_id = Some("t1".into());
        let mut step = StepEndThrowTeamMate::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
        assert!(game.defender_id.is_none());
        assert!(game.thrower_id.is_none());
        assert!(game.field_model.range_ruler.is_none());
    }

    #[test]
    fn restores_thrown_player_to_old_state_on_end_player_action() {
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "thrown".into();
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(3, 3));
        let old_state = PlayerState(0x101); // STANDING | ACTIVE
        let thrown_state = PlayerState(0x600); // some different state

        let mut step = StepEndThrowTeamMate::new();
        step.thrown_player_id = Some("thrown".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(7, 7));
        step.old_player_state = Some(old_state);
        step.thrown_player_state = Some(thrown_state);
        step.end_player_action = true;
        step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(game.field_model.player_state("thrown"), Some(old_state));
        assert_eq!(game.field_model.player_coordinate("thrown"), Some(FieldCoordinate::new(7, 7)));
    }

    #[test]
    fn restores_thrown_player_to_thrown_state_when_not_ending_action() {
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "thrown".into();
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("thrown", FieldCoordinate::new(3, 3));
        let thrown_state = PlayerState(0x600);

        let mut step = StepEndThrowTeamMate::new();
        step.thrown_player_id = Some("thrown".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(7, 7));
        step.thrown_player_state = Some(thrown_state);
        step.end_player_action = false;
        step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(game.field_model.player_state("thrown"), Some(thrown_state));
    }

    #[test]
    fn ball_coordinate_set_when_thrown_player_has_ball() {
        let mut game = make_game();
        let mut p = Player::default();
        p.id = "thrown".into();
        game.team_home.players.push(p);
        game.field_model.ball_coordinate = None;

        let mut step = StepEndThrowTeamMate::new();
        step.thrown_player_id = Some("thrown".into());
        step.thrown_player_coordinate = Some(FieldCoordinate::new(8, 6));
        step.thrown_player_has_ball = true;
        step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(game.field_model.ball_coordinate, Some(FieldCoordinate::new(8, 6)));
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_check_forgo_accepted() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::CheckForgo(true)));
        assert!(step.check_forgo);
    }

    #[test]
    fn set_parameter_thrown_player_has_ball_accepted() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(step.set_parameter(&StepParameter::ThrownPlayerHasBall(true)));
        assert!(step.thrown_player_has_ball);
    }

    #[test]
    fn set_parameter_unknown_rejected() {
        let mut step = StepEndThrowTeamMate::new();
        assert!(!step.set_parameter(&StepParameter::UsingChainsaw(true)));
    }
}
