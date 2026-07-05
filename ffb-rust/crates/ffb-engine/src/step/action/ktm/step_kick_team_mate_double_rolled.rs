/// 1:1 translation of com.fumbbl.ffb.server.step.action.ktm.StepKickTeamMateDoubleRolled (COMMON).
///
/// Handles the "double rolled" KTM failure path: restores the kicked player to their
/// original position and applies a crowd injury (always KNOCKED_OUT, no armor roll).
///
/// Expected preceding params: KICKED_PLAYER_ID, KICKED_PLAYER_STATE, KICKED_PLAYER_COORDINATE.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_KNOCKED_OUT};
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::injury::{InjuryContext, InjuryResult};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

pub struct StepKickTeamMateDoubleRolled {
    /// Java: fKickedPlayerId
    pub kicked_player_id: Option<String>,
    /// Java: fKickedPlayerState
    pub kicked_player_state: Option<PlayerState>,
    /// Java: fKickedPlayerCoordinate
    pub kicked_player_coordinate: Option<FieldCoordinate>,
}

impl StepKickTeamMateDoubleRolled {
    pub fn new() -> Self {
        Self {
            kicked_player_id: None,
            kicked_player_state: None,
            kicked_player_coordinate: None,
        }
    }
}

impl Default for StepKickTeamMateDoubleRolled {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickTeamMateDoubleRolled {
    fn id(&self) -> StepId { StepId::KickTeamMateDoubleRolled }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            StepParameter::KickedPlayerState(v) => { self.kicked_player_state = Some(*v); true }
            StepParameter::KickedPlayerCoordinate(v) => { self.kicked_player_coordinate = Some(*v); true }
            _ => false,
        }
    }
}

impl StepKickTeamMateDoubleRolled {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        let mut outcome = StepOutcome::next();

        // Java: if (kickedPlayer != null && coord != null && state.getId() > 0)
        if let (Some(ref kicked_id), Some(coord), Some(state)) = (
            &self.kicked_player_id,
            self.kicked_player_coordinate,
            self.kicked_player_state,
        ) {
            if state.id() > 0 {
                // Java: setPlayerCoordinate(kickedPlayer, coord); setPlayerState(defender, state)
                game.field_model.set_player_coordinate(kicked_id, coord);
                game.field_model.set_player_state(kicked_id, state);
                game.defender_id = None;

                // Java: InjuryTypeKTMCrowd — armor is auto-broken; injury = KNOCKED_OUT
                let mut ctx = InjuryContext::new(ApothecaryMode::ThrownPlayer);
                ctx.armor_broken = true;
                ctx.injury_roll = None; // crowd injury, no dice roll
                let mut injury = InjuryResult::new(ApothecaryMode::ThrownPlayer);
                injury.injury_context = ctx;
                injury.knocked_out = true;

                outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury)));

                // Java: if kicked coord == ball coord → ball_moving = true, publish END_TURN + SCATTER_BALL mode
                if game.field_model.ball_coordinate == Some(coord) {
                    game.field_model.ball_moving = true;
                    outcome = outcome
                        .publish(StepParameter::EndTurn(true))
                        .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
                }
            }
        }

        // Java: publish KICKED_PLAYER_COORDINATE(null) — avoids reset in end step
        outcome.publish(StepParameter::KickedPlayerCoordinate(FieldCoordinate::new(0, 0)))
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
    fn restores_player_position_and_publishes_injury() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("k1", FieldCoordinate::new(9, 9));
        game.defender_id = Some("k1".into());

        let mut step = StepKickTeamMateDoubleRolled::new();
        step.kicked_player_id = Some("k1".into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.kicked_player_coordinate = Some(coord);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_coordinate("k1"), Some(coord));
        assert_eq!(game.defender_id, None);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn ball_at_kicked_coord_triggers_scatter_and_end_turn() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("k2", coord);
        game.field_model.ball_coordinate = Some(coord);

        let mut step = StepKickTeamMateDoubleRolled::new();
        step.kicked_player_id = Some("k2".into());
        step.kicked_player_state = Some(PlayerState::new(PS_STANDING));
        step.kicked_player_coordinate = Some(coord);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall))));
    }

    #[test]
    fn invalid_state_skips_injury_but_still_publishes_coord_clear() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(3, 3);

        let mut step = StepKickTeamMateDoubleRolled::new();
        step.kicked_player_id = Some("k3".into());
        step.kicked_player_state = Some(PlayerState::new(0)); // id == 0 → skipped
        step.kicked_player_coordinate = Some(coord);

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
        // KICKED_PLAYER_COORDINATE(null) sentinel is always published
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickedPlayerCoordinate(_))));
    }

    #[test]
    fn set_parameter_kicked_player_id_accepted() {
        let mut step = StepKickTeamMateDoubleRolled::new();
        assert!(step.set_parameter(&StepParameter::KickedPlayerId(Some("kp".into()))));
        assert_eq!(step.kicked_player_id.as_deref(), Some("kp"));
    }

    #[test]
    fn set_parameter_kicked_player_state_accepted() {
        use ffb_model::enums::PS_STANDING;
        let mut step = StepKickTeamMateDoubleRolled::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::KickedPlayerState(state)));
        assert_eq!(step.kicked_player_state, Some(state));
    }
}
