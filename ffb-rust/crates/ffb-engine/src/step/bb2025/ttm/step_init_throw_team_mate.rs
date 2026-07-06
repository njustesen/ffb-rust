use ffb_model::enums::{PS_PICKED_UP, PS_STANDING, PlayerAction, PlayerState};
use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepAction, StepId, StepParameter};

/// Initialises the throw-team-mate sequence.
///
/// Java executeStep logic:
///   if endTurn:
///     publish END_TURN=true, CHECK_FORGO=true
///     GOTO gotoLabelOnEnd
///   else if endPlayerAction:
///     publish END_PLAYER_ACTION=true
///     GOTO gotoLabelOnEnd
///   else:
///     if thrownPlayerId provided:
///       if targetCoordinate provided:
///         game.setPassCoordinate(target)
///         game.fieldModel.setRangeRuler(UtilRangeRuler.createRangeRuler(...))
///         if rangeRuler != null -> NEXT_STEP
///       else (player chosen, no target yet):
///         game.setDefenderId(thrownPlayerId)
///         oldPlayerState = fieldModel.getPlayerState(defender)
///         thrownPlayerState = oldPlayerState.changeBase(PICKED_UP)
///         publish THROWN_PLAYER_ID, THROWN_PLAYER_STATE, OLD_DEFENDER_STATE,
///                THROWN_PLAYER_COORDINATE, THROWN_PLAYER_HAS_BALL
///         fieldModel.setPlayerState(defender, thrownPlayerState)
///         changePlayerAction(actingPlayer, KICK_TEAM_MATE or THROW_TEAM_MATE)
///   (implicit) if nothing was done: step waits (Continue) for client commands
///
/// Command handling (Java handleCommand):
///   CLIENT_THROW_TEAM_MATE:
///     if target already known (thrownPlayerId provided): update targetCoordinate -> EXECUTE_STEP
///     else: set thrownPlayerId from command -> EXECUTE_STEP
///   CLIENT_ACTING_PLAYER:
///     if playerId provided: changePlayerAction -> EXECUTE_STEP
///     else: endPlayerAction=true -> EXECUTE_STEP
///   CLIENT_END_TURN:
///     endTurn=true -> EXECUTE_STEP
///
/// Unported utilities:
///   TODO: UtilRangeRuler.createRangeRuler (range ruler for TTM throw)
///   TODO: game.setPassCoordinate / game.setDefenderId / game.getDefender
///   TODO: game.fieldModel.setPlayerState / getPlayerState / getPlayerCoordinate
///   TODO: UtilServerSteps.changePlayerAction
///   ✓ coordinate.transform() applied for away-team client commands
///   TODO: PICKED_UP base state
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.ttm.StepInitThrowTeamMate`.
pub struct StepInitThrowTeamMate {
    /// Java: gotoLabelOnEnd (mandatory init param GOTO_LABEL_ON_END)
    pub goto_label_on_end: String,
    /// Java: thrownPlayerId (optional init param THROWN_PLAYER_ID, or from CLIENT_THROW_TEAM_MATE)
    pub thrown_player_id: Option<String>,
    /// Java: targetCoordinate (optional init param, or from CLIENT_THROW_TEAM_MATE)
    pub target_coordinate: Option<FieldCoordinate>,
    /// Java: endTurn (set by CLIENT_END_TURN)
    pub end_turn: bool,
    /// Java: endPlayerAction (set by CLIENT_ACTING_PLAYER with null playerId)
    pub end_player_action: bool,
    /// Java: kicked (optional init param IS_KICKED_PLAYER)
    pub kicked: bool,
}

impl StepInitThrowTeamMate {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            thrown_player_id: None,
            target_coordinate: None,
            end_turn: false,
            end_player_action: false,
            kicked: false,
        }
    }
}

impl Default for StepInitThrowTeamMate {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepInitThrowTeamMate {
    fn id(&self) -> StepId { StepId::InitThrowTeamMate }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_THROW_TEAM_MATE -> update thrownPlayerId / targetCoordinate -> EXECUTE_STEP
        // Java: CLIENT_ACTING_PLAYER -> endPlayerAction or changePlayerAction -> EXECUTE_STEP
        // Java: CLIENT_END_TURN -> endTurn=true -> EXECUTE_STEP
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            Action::ThrowTeamMate { player_id, coord } => {
                if self.thrown_player_id.is_some() {
                    // target chosen after player; transform if from away-team client
                    let is_home = game.acting_player.player_id.as_deref()
                        .map(|id| game.team_home.player(id).is_some())
                        .unwrap_or(game.home_playing);
                    self.target_coordinate = Some(if is_home { *coord } else { coord.transform() });
                } else {
                    self.thrown_player_id = Some(player_id.clone());
                }
            }
            Action::KickTeamMate { player_id, coord } => {
                if self.thrown_player_id.is_some() {
                    // target chosen after player; transform if from away-team client
                    let is_home = game.acting_player.player_id.as_deref()
                        .map(|id| game.team_home.player(id).is_some())
                        .unwrap_or(game.home_playing);
                    self.target_coordinate = Some(if is_home { *coord } else { coord.transform() });
                } else {
                    self.thrown_player_id = Some(player_id.clone());
                }
            }
            Action::SelectPlayer { player_id
} => {
                // CLIENT_ACTING_PLAYER with playerId -> change action (not relevant for random agent)
                let _ = player_id;
            }
            Action::Acknowledge => {
                // CLIENT_ACTING_PLAYER with null playerId -> endPlayerAction
                self.end_player_action = true;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepInitThrowTeamMate {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        if self.end_turn {
            // Java: publishParameter(END_TURN, true); publishParameter(CHECK_FORGO, true)
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }
        if let Some(pid) = &self.thrown_player_id {
            if let Some(target) = &self.target_coordinate {
                // Java: game.setPassCoordinate(target)
                game.pass_coordinate = Some(*target);
                // client-only: game.fieldModel.setRangeRuler(UtilRangeRuler.createRangeRuler(...)) — range ruler is client-side display
                return StepOutcome::next();
            }
            // Player selected, no target yet: wire up thrown player state
            // Java: game.setDefenderId(thrownPlayerId)
            game.defender_id = Some(pid.clone());
            // Java: oldPlayerState = fieldModel.getPlayerState(defender)
            let old_state = game.field_model.player_state(pid)
                .unwrap_or(PlayerState::new(PS_STANDING));
            // Java: thrownPlayerState = oldPlayerState.changeBase(PICKED_UP)
            let thrown_state = old_state.change_base(PS_PICKED_UP);
            // Java: fieldModel.setPlayerState(defender, thrownPlayerState)
            game.field_model.set_player_state(pid, thrown_state);
            let coord = game.field_model.player_coordinate(pid);
            let has_ball = coord
                .map(|c| game.field_model.ball_coordinate == Some(c) && !game.field_model.ball_moving)
                .unwrap_or(false);
            // Java: UtilServerSteps.changePlayerAction(actingPlayer, KICK_TEAM_MATE or THROW_TEAM_MATE)
            if self.kicked {
                game.acting_player.player_action = Some(PlayerAction::KickTeamMate);
            } else {
                game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
            }
            return StepOutcome::cont()
                .publish(StepParameter::ThrownPlayerId(Some(pid.clone())))
                .publish(StepParameter::ThrownPlayerState(thrown_state))
                .publish(StepParameter::OldDefenderState(old_state))
                .publish(StepParameter::ThrownPlayerCoordinate(coord))
                .publish(StepParameter::ThrownPlayerHasBall(has_ball));
        }
        // No player chosen yet: wait for CLIENT_THROW_TEAM_MATE
        StepOutcome::cont()
    }
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
    fn start_no_player_waits() {
        let mut game = make_game();
        let mut step = StepInitThrowTeamMate::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_command_goes_to_label() {
        let mut game = make_game();
        let mut step = StepInitThrowTeamMate::new("end_label".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
    }

    #[test]
    fn end_player_action_goes_to_label() {
        let mut game = make_game();
        let mut step = StepInitThrowTeamMate::new("end_label".into());
        // Java CLIENT_ACTING_PLAYER (null player id) mapped to Acknowledge
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
    }

    #[test]
    fn throw_team_mate_command_stores_player_id() {
        let mut game = make_game();
        let mut step = StepInitThrowTeamMate::new("end".into());
        let coord = FieldCoordinate { x: 5, y: 5 };
        step.handle_command(&Action::ThrowTeamMate { player_id: "p1".into(), coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.thrown_player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn throw_team_mate_command_stores_target_when_player_known() {
        let mut game = make_game();
        let mut step = StepInitThrowTeamMate::new("end".into());
        step.thrown_player_id = Some("p1".into());
        let coord = FieldCoordinate { x: 5, y: 5 };
        step.handle_command(&Action::ThrowTeamMate { player_id: "p1".into(), coord }, &mut game, &mut GameRng::new(0));
        assert_eq!(step.target_coordinate, Some(coord));
    }
}
