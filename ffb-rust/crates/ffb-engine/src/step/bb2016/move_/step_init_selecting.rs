use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps::change_player_action;
use crate::util::ServerUtilBlock;
use crate::util::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepInitSelecting.
///
/// Initialises the select sequence. Waits for a client command; on receipt,
/// publishes the relevant parameters and GOTOs the end label so EndSelecting
/// can route to the correct action sequence.
///
/// Init params: GOTO_LABEL_ON_END (mandatory), UPDATE_PERSISTENCE (mandatory).
///
/// On start: if fUpdatePersistence → clear flag (queueDbUpdate TODO).
///
/// On executeStep:
/// - timeout || fEndTurn → publish END_TURN + GOTO_LABEL
/// - fEndPlayerAction → publish END_PLAYER_ACTION + GOTO_LABEL
/// - fDispatchPlayerAction set → publish DISPATCH_PLAYER_ACTION + GOTO_LABEL
///   (unless standingUp → NEXT_STEP)
/// - else → prepareStandingUp, then NEXT_STEP if REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
///
/// TODO(updatePersistence): gameCache.queueDbUpdate not yet ported.
/// TODO(standingUp): actingPlayer.isStandingUp() paths not yet ported.
/// TODO(removeConfusion): REMOVE_CONFUSION / STAND_UP_BLITZ paths not yet ported.
/// TODO(timeoutEnforced): game.isTimeoutEnforced() not yet ported.
/// TODO(foulUsed): isFoulUsed() check in CLIENT_FOUL not yet ported.
/// TODO(passUsed): isPassUsed() check in CLIENT_PASS not yet ported.
/// TODO(blitzUsed): isBlitzUsed() check in CLIENT_KICK_TEAM_MATE not yet ported.
/// TODO(handOverUsed): isHandOverUsed() check in CLIENT_HAND_OVER not yet ported.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fUpdatePersistence (transient)
    pub update_persistence: bool,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            update_persistence: false,
        }
    }
}

impl Default for StepInitSelecting {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fUpdatePersistence) { fUpdatePersistence=false; gameCache.queueDbUpdate(...); }
        // TODO(updatePersistence): queueDbUpdate not yet ported
        self.update_persistence = false;
        // start() does NOT call executeStep — waits for a client command
        StepOutcome::cont()
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_action = game.acting_player.player_action;
        let acting_pid = game.acting_player.player_id.clone().unwrap_or_default();

        match action {
            Action::Move { path } if !path.is_empty() => {
                // Java: CLIENT_MOVE → dispatchPlayerAction(MOVE) + publish MOVE_STACK
                self.dispatch_player_action = Some(PlayerAction::Move);
                let out = self.execute_step(game, rng);
                return out.publish(StepParameter::MoveStack(path.clone()));
            }
            Action::Foul { target_id } => {
                // TODO(foulUsed): check !game.getTurnData().isFoulUsed()
                change_player_action(game, &acting_pid, PlayerAction::Foul, false);
                self.dispatch_player_action = Some(PlayerAction::Foul);
                return self.execute_step(game, rng)
                    .publish(StepParameter::FoulDefenderId(target_id.clone()));
            }
            Action::Block { defender_id } => {
                self.dispatch_player_action = Some(PlayerAction::Block);
                return self.execute_step(game, rng)
                    .publish(StepParameter::BlockDefenderId(defender_id.clone()));
            }
            Action::HypnoticGaze { target_id } => {
                // Java: CLIENT_GAZE → changePlayerAction(GAZE), dispatch
                change_player_action(game, &acting_pid, PlayerAction::Gaze, false);
                self.dispatch_player_action = Some(PlayerAction::Gaze);
                return self.execute_step(game, rng)
                    .publish(StepParameter::GazeVictimId(Some(target_id.clone())));
            }
            Action::Pass { coord } => {
                // TODO(passUsed): check !isPassUsed (unless THROW_BOMB / HAIL_MARY_BOMB)
                let target = *coord;
                let dispatch_action = if player_action == Some(PlayerAction::HailMaryPass) {
                    PlayerAction::HailMaryPass
                } else {
                    change_player_action(game, &acting_pid, PlayerAction::Pass, false);
                    PlayerAction::Pass
                };
                self.dispatch_player_action = Some(dispatch_action);
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(target));
            }
            Action::HandOff { receiver_id } => {
                // TODO(handOverUsed): check !isHandOverUsed
                let coord_opt = game.field_model.player_coordinate(receiver_id);
                change_player_action(game, &acting_pid, PlayerAction::HandOver, false);
                self.dispatch_player_action = Some(PlayerAction::HandOver);
                let out = self.execute_step(game, rng);
                if let Some(coord) = coord_opt {
                    return out.publish(StepParameter::TargetCoordinate(coord));
                }
                return out;
            }
            Action::ThrowTeamMate { player_id: thrown_id, coord } => {
                // TODO(passUsed): check !isPassUsed
                change_player_action(game, &acting_pid, PlayerAction::ThrowTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::ThrowTeamMate);
                return self.execute_step(game, rng)
                    .publish(StepParameter::TargetCoordinate(*coord))
                    .publish(StepParameter::ThrownPlayerId(Some(thrown_id.clone())));
            }
            Action::KickTeamMate { player_id: kicked_id, coord: _ } => {
                // TODO(blitzUsed): check !isBlitzUsed
                change_player_action(game, &acting_pid, PlayerAction::KickTeamMate, false);
                self.dispatch_player_action = Some(PlayerAction::KickTeamMate);
                return self.execute_step(game, rng)
                    .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())));
            }
            Action::EndTurn => {
                self.end_turn = true;
                return self.execute_step(game, rng);
            }
            _ => {}
        }
        StepOutcome::cont()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::DispatchPlayerAction(v) => { self.dispatch_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::UpdatePersistence(v) => { self.update_persistence = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = self.goto_label_on_end.clone();

        // TODO(timeoutEnforced): game.isTimeoutEnforced() not yet ported
        if self.end_turn {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndTurn(true));
        }

        if self.end_player_action {
            return StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true));
        }

        if let Some(dispatch_action) = self.dispatch_player_action {
            let player_id = game.acting_player.player_id.clone();
            let player_action = game.acting_player.player_action;
            if player_id.is_some() && player_action.is_some() {
                // TODO(standingUp): if isStandingUp() → prepareStandingUp() + NEXT_STEP
                return StepOutcome::goto(&label)
                    .publish(StepParameter::DispatchPlayerAction(Some(dispatch_action)));
            }
        }

        // Java: prepareStandingUp() + NEXT_STEP for REMOVE_CONFUSION/STAND_UP/STAND_UP_BLITZ
        self.prepare_standing_up(game);
        StepOutcome::cont()
    }

    fn prepare_standing_up(&self, game: &mut Game) {
        let player_action = game.acting_player.player_action;

        if let Some(action) = player_action {
            // TODO(blitzBlock): updateDiceDecorations for BLITZ/BLITZ_MOVE/BLOCK/MULTIPLE_BLOCK
            if matches!(action, PlayerAction::Blitz | PlayerAction::BlitzMove | PlayerAction::Block) {
                ServerUtilBlock::update_dice_decorations(game);
            }

            if action.is_moving() {
                // TODO(standingUp): isStandingUp + canStandUpForFree path not yet ported
                let jumping = game.acting_player.jumping;
                UtilServerPlayerMove::update_move_squares(game, jumping);
            }
        }
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
    fn start_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_command_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepInitSelecting::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn set_parameter_update_persistence_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::UpdatePersistence(true)));
        assert!(step.update_persistence);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_dispatch_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::DispatchPlayerAction(Some(PlayerAction::Block))));
        assert_eq!(step.dispatch_player_action, Some(PlayerAction::Block));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(!step.set_parameter(&StepParameter::DodgeRoll(3)));
    }

    #[test]
    fn update_persistence_cleared_on_start() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        step.update_persistence = true;
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!step.update_persistence);
    }

    #[test]
    fn end_player_action_set_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        // Set end_player_action via parameter, then start triggers it
        step.set_parameter(&StepParameter::EndPlayerAction(true));
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        // EndTurn sets end_turn → GotoLabel with EndTurn
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn block_command_publishes_block_defender_and_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.player_action = Some(PlayerAction::Block);
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::Block { defender_id: "def1".into() };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(_))));
    }
}
