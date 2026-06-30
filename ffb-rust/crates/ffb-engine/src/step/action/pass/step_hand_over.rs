/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepHandOver (COMMON).
///
/// Handles a hand-over (hand-off) of the ball in the passing sequence.
///
/// Expected preceding param: CATCHER_ID.
/// Publishes CATCH_SCATTER_THROW_IN_MODE = CatchHandOff if the catcher is adjacent.
/// Publishes END_PLAYER_ACTION = true always.
/// Always returns NEXT_STEP.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

pub struct StepHandOver {
    /// Java: fCatcherId — set by preceding step parameter.
    pub catcher_id: Option<String>,
}

impl StepHandOver {
    pub fn new() -> Self {
        Self { catcher_id: None }
    }
}

impl Default for StepHandOver {
    fn default() -> Self { Self::new() }
}

impl Step for StepHandOver {
    fn id(&self) -> StepId { StepId::HandOver }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepHandOver {
    fn execute_step(&self, game: &mut Game) -> StepOutcome {
        // Java: game.getFieldModel().setBallMoving(true)
        game.field_model.ball_moving = true;
        // Java: game.setPassCoordinate(null)
        game.pass_coordinate = None;

        // Java: thrower = game.getActingPlayer().getPlayer()
        //       throwerCoordinate = game.getFieldModel().getPlayerCoordinate(thrower)
        let thrower_coord = game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        // Java: catcher = game.getPlayerById(fCatcherId)
        //       catcherCoordinate = game.getFieldModel().getPlayerCoordinate(catcher)
        let catcher_coord = self.catcher_id.as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        let mut out = StepOutcome::next();

        if let (Some(tc), Some(cc)) = (thrower_coord, catcher_coord) {
            // Java: if (throwerCoordinate != null && throwerCoordinate.isAdjacent(catcherCoordinate))
            if tc.is_adjacent(cc) {
                // Java: game.getFieldModel().setBallCoordinate(catcherCoordinate)
                game.field_model.ball_coordinate = Some(cc);
                // Java: publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_HAND_OFF)
                out = out.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchHandOff));
            }
        }

        // Java: publishParameter(END_PLAYER_ACTION, true)
        out.publish(StepParameter::EndPlayerAction(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{CatchScatterThrowInMode, StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn adjacent_catcher_sets_ball_and_publishes_catch_mode() {
        let mut game = make_game();
        let thrower_coord = FieldCoordinate::new(5, 5);
        let catcher_coord = FieldCoordinate::new(6, 5); // adjacent
        game.field_model.set_player_coordinate("thrower", thrower_coord);
        game.field_model.set_player_coordinate("catcher", catcher_coord);
        game.acting_player.player_id = Some("thrower".into());
        let mut step = StepHandOver::new();
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.ball_coordinate, Some(catcher_coord));
        assert!(game.field_model.ball_moving);
        assert!(game.pass_coordinate.is_none());

        let has_catch_mode = out.published.iter().any(|p| matches!(
            p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchHandOff)
        ));
        assert!(has_catch_mode, "should publish CatchHandOff mode");

        let has_end_action = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_end_action, "should publish EndPlayerAction(true)");
    }

    #[test]
    fn non_adjacent_catcher_does_not_set_ball_coord() {
        let mut game = make_game();
        let thrower_coord = FieldCoordinate::new(5, 5);
        let catcher_coord = FieldCoordinate::new(8, 5); // not adjacent
        game.field_model.set_player_coordinate("thrower", thrower_coord);
        game.field_model.set_player_coordinate("catcher", catcher_coord);
        game.acting_player.player_id = Some("thrower".into());
        game.field_model.ball_coordinate = None;
        let mut step = StepHandOver::new();
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.field_model.ball_coordinate.is_none(), "ball should not move");

        let has_catch_mode = out.published.iter().any(|p| matches!(
            p, StepParameter::CatchScatterThrowInMode(_)
        ));
        assert!(!has_catch_mode, "should NOT publish CatchHandOff when not adjacent");

        let has_end_action = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_end_action, "should always publish EndPlayerAction(true)");
    }

    #[test]
    fn no_catcher_id_publishes_end_player_action_only() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("thrower", FieldCoordinate::new(5, 5));
        game.acting_player.player_id = Some("thrower".into());
        let mut step = StepHandOver::new();
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        let has_end_action = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_end_action);
    }

    #[test]
    fn no_thrower_coord_publishes_end_player_action_only() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("catcher", FieldCoordinate::new(6, 5));
        game.acting_player.player_id = Some("thrower".into()); // thrower has no coordinate
        let mut step = StepHandOver::new();
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        let has_end_action = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_end_action);
    }

    #[test]
    fn sets_ball_moving_and_clears_pass_coordinate() {
        let mut game = make_game();
        game.field_model.ball_moving = false;
        game.pass_coordinate = Some(FieldCoordinate::new(3, 3));
        game.acting_player.player_id = Some("t".into());
        let mut step = StepHandOver::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
        assert!(game.pass_coordinate.is_none());
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = StepHandOver::new();
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }
}
