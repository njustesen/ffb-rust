/// 1:1 translation of com.fumbbl.ffb.server.step.action.ktm.StepEndKickTeamMate (COMMON).
///
/// Final cleanup step of the KTM sequence. Resets the kicked player to their original
/// position if the sequence was aborted before they landed. Clears pass_coordinate.
///
/// Stub: EndPlayerAction/Select sequence push omitted pending generator integration.
/// Stub: cleanupStepStack(END_MOVING) omitted.
use ffb_model::enums::PlayerState;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepEndKickTeamMate {
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: fKickedPlayerId (also set by THROWN_PLAYER_ID)
    pub kicked_player_id: Option<String>,
    /// Java: fKickedPlayerState (also set by THROWN_PLAYER_STATE)
    pub kicked_player_state: Option<PlayerState>,
    /// Java: fKickedPlayerHasBall (also set by THROWN_PLAYER_HAS_BALL)
    pub kicked_player_has_ball: bool,
    /// Java: fKickedPlayerCoordinate (also set by THROWN_PLAYER_COORDINATE)
    pub kicked_player_coordinate: Option<FieldCoordinate>,
}

impl StepEndKickTeamMate {
    pub fn new() -> Self {
        Self {
            end_turn: false,
            end_player_action: false,
            kicked_player_id: None,
            kicked_player_state: None,
            kicked_player_has_ball: false,
            kicked_player_coordinate: None,
        }
    }
}

impl Default for StepEndKickTeamMate {
    fn default() -> Self { Self::new() }
}

impl Step for StepEndKickTeamMate {
    fn id(&self) -> StepId { StepId::EndKickTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_ACTING_PLAYER → push Select, NEXT_STEP_AND_REPEAT
            // Stub: Select sequence not yet translated; return NextStep.
            Action::ActivatePlayer { .. } => StepOutcome::next(),
            _ => self.execute_step(game),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            // Java: both KICKED_PLAYER_ID and THROWN_PLAYER_ID map to kicked_player_id
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::ThrownPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            // Java: both KICKED_PLAYER_STATE and THROWN_PLAYER_STATE
            StepParameter::KickedPlayerState(v) => { self.kicked_player_state = Some(*v); true }
            StepParameter::ThrownPlayerState(v) => { self.kicked_player_state = Some(*v); true }
            // Java: both KICKED_PLAYER_HAS_BALL and THROWN_PLAYER_HAS_BALL
            StepParameter::KickedPlayerHasBall(v) => { self.kicked_player_has_ball = *v; true }
            StepParameter::ThrownPlayerHasBall(v) => { self.kicked_player_has_ball = *v; true }
            // Java: both KICKED_PLAYER_COORDINATE and THROWN_PLAYER_COORDINATE
            StepParameter::KickedPlayerCoordinate(v) => { self.kicked_player_coordinate = *v; true }
            StepParameter::ThrownPlayerCoordinate(v) => { self.kicked_player_coordinate = v.clone(); true }
            _ => false,
        }
    }
}

impl StepEndKickTeamMate {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: game.setPassCoordinate(null)
        game.pass_coordinate = None;
        // Java: game.getFieldModel().setRangeRuler(null) — not yet implemented, skip.

        // Java: reset thrown/kicked player if sequence was aborted before landing
        if let (Some(ref kicked_id), Some(coord), Some(state)) = (
            &self.kicked_player_id,
            self.kicked_player_coordinate,
            self.kicked_player_state,
        ) {
            if state.id() > 0 {
                game.field_model.set_player_coordinate(kicked_id, coord);
                game.field_model.set_player_state(kicked_id, state);
                if self.kicked_player_has_ball {
                    game.field_model.ball_coordinate = Some(coord);
                }
            }
        }

        // Java: cleanupStepStack(END_MOVING) + pushSequence(EndPlayerAction) — stub: omitted.
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn clears_pass_coordinate() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(5, 5));

        let mut step = StepEndKickTeamMate::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn restores_kicked_player_position_when_state_valid() {
        let mut game = make_game();
        let original_coord = FieldCoordinate::new(3, 4);
        game.field_model.set_player_coordinate("k1", FieldCoordinate::new(9, 9));
        game.field_model.set_player_state("k1", PlayerState::new(PS_STANDING));

        let mut step = StepEndKickTeamMate::new();
        step.kicked_player_id = Some("k1".into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.kicked_player_coordinate = Some(original_coord);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_coordinate("k1"), Some(original_coord));
    }

    #[test]
    fn restores_ball_when_player_had_ball() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(6, 6);

        let mut step = StepEndKickTeamMate::new();
        step.kicked_player_id = Some("k2".into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.kicked_player_coordinate = Some(coord);
        step.kicked_player_has_ball = true;

        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(coord));
    }

    #[test]
    fn skips_restore_when_state_zero() {
        let mut game = make_game();
        let original_coord = FieldCoordinate::new(2, 2);
        game.field_model.set_player_coordinate("k3", original_coord);

        let mut step = StepEndKickTeamMate::new();
        step.kicked_player_id = Some("k3".into());
        step.kicked_player_state = Some(PlayerState::new(0)); // id == 0 → skip
        step.kicked_player_coordinate = Some(FieldCoordinate::new(9, 9));

        step.start(&mut game, &mut GameRng::new(0));
        // Coordinate should NOT be changed
        assert_eq!(game.field_model.player_coordinate("k3"), Some(original_coord));
    }

    #[test]
    fn thrown_player_params_also_accepted() {
        let mut step = StepEndKickTeamMate::new();
        step.set_parameter(&StepParameter::ThrownPlayerId(Some("tp1".into())));
        step.set_parameter(&StepParameter::ThrownPlayerState(PlayerState::new(PS_STANDING)));
        step.set_parameter(&StepParameter::ThrownPlayerHasBall(true));
        assert_eq!(step.kicked_player_id.as_deref(), Some("tp1"));
        assert!(step.kicked_player_state.is_some());
        assert!(step.kicked_player_has_ball);
    }
}
