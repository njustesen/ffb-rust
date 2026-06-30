use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2016::KickTeamMate;
use crate::step::generator::bb2016::kick_team_mate::KickTeamMateParams;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepInitMoving.
///
/// Initialises the move sequence. Handles client move commands and dispatches
/// player actions.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), GAZE_VICTIM_ID (optional),
///              MOVE_STACK (optional).
///
/// On start/execute:
/// - fEndTurn → publish END_TURN + GOTO_LABEL_ON_END
/// - fEndPlayerAction → publish END_PLAYER_ACTION + GOTO_LABEL_ON_END
/// - fGazeVictimId → setDefenderId + setPlayerAction(GAZE) + NEXT_STEP
/// - fKickedPlayerId → push KickTeamMate sequence + publish KICKED_PLAYER_ID + NR_OF_DICE + NEXT_STEP
/// - fMoveStack provided → publish COORDINATE_FROM / COORDINATE_TO / MOVE_STACK (shifted),
///   set dodging/goingForIt/hasMoved, update turnStarted, NEXT_STEP
///
/// TODO(blitzUsed): turnData.setBlitzUsed for BLITZ_MOVE/KTM_MOVE actions not yet ported.
/// TODO(foulUsed): turnData.setFoulUsed for FOUL_MOVE not yet ported.
/// TODO(passUsed): turnData.setPassUsed for PASS_MOVE/TTM_MOVE not yet ported.
/// TODO(handOverUsed): turnData.setHandOverUsed for HAND_OVER_MOVE not yet ported.
/// TODO(concessionPossible): game.setConcessionPossible(false) not yet ported.
/// TODO(moveSquareLookup): getMoveSquare(coordinateTo) not yet ported — dodging/goesForIt hardcoded.
/// TODO(validMove): UtilServerPlayerMove.isValidMove check not yet ported in handle_command.
/// TODO(dispatchPlayerAction): GOTO_LABEL_AND_REPEAT from handle_command not yet ported (returns GOTO_LABEL).
pub struct StepInitMoving {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fMoveStack
    pub move_stack: Vec<FieldCoordinate>,
    /// Java: fGazeVictimId
    pub gaze_victim_id: Option<String>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fKickedPlayerId
    pub kicked_player_id: Option<String>,
    /// Java: fNumDice
    pub num_dice: i32,
}

impl StepInitMoving {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            move_stack: Vec::new(),
            gaze_victim_id: None,
            end_turn: false,
            end_player_action: false,
            kicked_player_id: None,
            num_dice: 0,
        }
    }
}

impl Default for StepInitMoving {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitMoving {
    fn id(&self) -> StepId { StepId::InitMoving }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        match action {
            Action::Move { path } if !path.is_empty() => {
                if self.move_stack.is_empty() {
                    // Java: UtilServerPlayerMove.fetchMoveStack(moveCommand) → fMoveStack
                    self.move_stack = path.clone();
                    return self.execute_step(game, rng);
                }
            }
            Action::Block { .. } => {
                // Java: BLITZ_MOVE + !hasBlocked → dispatchPlayerAction(BLITZ)
                if player_action == Some(PlayerAction::BlitzMove)
                    && !game.acting_player.has_blocked
                    && self.move_stack.is_empty()
                {
                    return self.dispatch_player_action(PlayerAction::Blitz);
                }
            }
            Action::Foul { .. } => {
                if player_action == Some(PlayerAction::FoulMove)
                    && !game.acting_player.has_fouled
                    && self.move_stack.is_empty()
                {
                    return self.dispatch_player_action(PlayerAction::Foul);
                }
            }
            Action::HandOff { .. } => {
                if player_action == Some(PlayerAction::HandOverMove)
                    || player_action == Some(PlayerAction::HandOver)
                {
                    return self.dispatch_player_action(PlayerAction::HandOver);
                }
            }
            Action::Pass { .. } => {
                if player_action == Some(PlayerAction::PassMove)
                    || player_action == Some(PlayerAction::Pass)
                {
                    return self.dispatch_player_action(PlayerAction::Pass);
                }
                if player_action == Some(PlayerAction::HailMaryPass) {
                    return self.dispatch_player_action(PlayerAction::HailMaryPass);
                }
            }
            Action::ThrowTeamMate { .. } => {
                if player_action == Some(PlayerAction::ThrowTeamMateMove) && self.move_stack.is_empty() {
                    return self.dispatch_player_action(PlayerAction::ThrowTeamMate);
                }
            }
            Action::KickTeamMate { player_id, coord: _ } => {
                if player_action == Some(PlayerAction::KickTeamMateMove) && self.move_stack.is_empty() {
                    self.kicked_player_id = Some(player_id.clone());
                    // Java: fNumDice = kickTeamMateCommand.getNumDice()
                    return self.execute_step(game, rng);
                }
            }
            Action::HypnoticGaze { target_id } => {
                // Java: CLIENT_GAZE → fGazeVictimId = victimId, EXECUTE_STEP
                self.gaze_victim_id = Some(target_id.clone());
                return self.execute_step(game, rng);
            }
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        // Still waiting
        StepOutcome::cont()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::MoveStack(v) => { self.move_stack = v.clone(); true }
            StepParameter::GazeVictimId(v) => { self.gaze_victim_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepInitMoving {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }

        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(ref victim_id) = self.gaze_victim_id.clone() {
            // Java: game.setDefenderId(fGazeVictimId)
            game.defender_id = Some(victim_id.clone());
            game.acting_player.player_action = Some(PlayerAction::Gaze);
            return StepOutcome::next();
        }

        if let Some(ref kicked_id) = self.kicked_player_id.clone() {
            let seq = KickTeamMate::build_sequence(&KickTeamMateParams::default());
            return StepOutcome::next()
                .push_seq(seq)
                .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())))
                .publish(StepParameter::NrOfDice(self.num_dice));
        }

        if !self.move_stack.is_empty() {
            // Java: coordinateTo = fMoveStack[0]; newMoveStack = fMoveStack[1..]
            let coordinate_to = self.move_stack[0];
            let new_move_stack: Vec<FieldCoordinate> = self.move_stack[1..].to_vec();

            // Only proceed if coordinateTo is on the field
            if FieldCoordinateBounds::FIELD.is_in_bounds(coordinate_to) {
                let coordinate_from = game.acting_player.player_id.as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id))
                    .unwrap_or(FieldCoordinate::new(0, 0));

                // TODO(moveSquareLookup): getMoveSquare(coordinateTo) not yet ported
                // TODO(dodging): actingPlayer.setDodging not yet ported — no dodging field on ActingPlayer
                game.acting_player.goes_for_it = false; // will be set properly when MoveSquare is ported
                game.acting_player.has_moved = true;

                // TODO(turnStarted): game.getTurnData().setTurnStarted(true) not yet ported
                // TODO(blitzUsed/foulUsed/passUsed/handOverUsed): turn data flags not yet ported
                // TODO(concessionPossible): game.setConcessionPossible(false) not yet ported

                return StepOutcome::next()
                    .publish(StepParameter::MoveStack(new_move_stack))
                    .publish(StepParameter::CoordinateFrom(coordinate_from))
                    .publish(StepParameter::CoordinateTo(coordinate_to));
            }
        }

        // Still waiting for a command
        StepOutcome::cont()
    }

    fn dispatch_player_action(&self, action: PlayerAction) -> StepOutcome {
        // Java: publishParameter(DISPATCH_PLAYER_ACTION, pPlayerAction)
        //       setNextAction(GOTO_LABEL_AND_REPEAT, fGotoLabelOnEnd)
        // TODO(gotoLabelAndRepeat): StepAction::GotoLabelAndRepeat not yet wired — use GotoLabel
        let label = self.goto_label_on_end.clone();
        StepOutcome::goto(&label)
            .publish(StepParameter::DispatchPlayerAction(Some(action)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn start_with_no_move_stack_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_publishes_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_publishes_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn gaze_victim_sets_defender_and_returns_next_step() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        step.gaze_victim_id = Some("victim".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.defender_id.as_deref(), Some("victim"));
    }

    #[test]
    fn move_stack_with_one_coord_publishes_from_to() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepInitMoving::new("end".into());
        step.move_stack = vec![FieldCoordinate::new(6, 5)];
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateTo(_))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CoordinateFrom(_))));
    }

    #[test]
    fn move_stack_is_shifted_when_published() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepInitMoving::new("end".into());
        let remaining = FieldCoordinate::new(7, 5);
        step.move_stack = vec![FieldCoordinate::new(6, 5), remaining];
        let out = step.start(&mut game, &mut GameRng::new(0));
        let stack_param = out.published.iter().find(|p| matches!(p, StepParameter::MoveStack(_)));
        if let Some(StepParameter::MoveStack(s)) = stack_param {
            assert_eq!(s.len(), 1);
            assert_eq!(s[0], remaining);
        } else {
            panic!("MoveStack not published");
        }
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        let mut step = StepInitMoving::new("end".into());
        let stack = vec![FieldCoordinate::new(5, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack.clone())));
        assert_eq!(step.move_stack, stack);
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepInitMoving::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitMoving::new("end".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn end_turn_command_publishes_and_gotos() {
        let mut game = make_game();
        let mut step = StepInitMoving::new("end".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }
}
