use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{CatchScatterThrowInMode, StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.pass.StepHandOver.
///
/// Hand-off action step.  Flow:
///  1. Set ball moving, clear passCoordinate.
///  2. If thrower and catcher are adjacent: move ball to catcher's square,
///     publish `CatchScatterThrowInMode::CatchHandOff`.
///  3. Unless thrower has `canMoveAfterHandOff`: publish `EndPlayerAction(true)`.
///  4. Always → NEXT_STEP.
///
/// Expects stepParameter `CatcherId` from a preceding step.
/// Publishes: `CatchScatterThrowInMode`, `EndPlayerAction`.
///
/// Checks `NamedProperties::CAN_MOVE_AFTER_HAND_OFF` on the thrower via `has_skill_property`.
pub struct StepHandOver {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    // AbstractStepWithReRoll fields
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepHandOver {
    pub fn new() -> Self {
        Self { catcher_id: None, re_rolled_action: None, re_roll_source: None }
    }
}

impl Default for StepHandOver {
    fn default() -> Self { Self::new() }
}

impl Step for StepHandOver {
    fn id(&self) -> StepId { StepId::HandOver }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepHandOver {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: game.getFieldModel().setBallMoving(true)
        game.field_model.ball_moving = true;
        // Java: game.setPassCoordinate(null)
        game.pass_coordinate = None;

        // Resolve thrower position
        let thrower_coord = game
            .thrower_id
            .as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        // Resolve catcher position
        let catcher_coord = self
            .catcher_id
            .as_deref()
            .and_then(|id| game.field_model.player_coordinate(id));

        let mut outcome = StepOutcome::next();

        // Java: if throwerCoordinate != null && throwerCoordinate.isAdjacent(catcherCoordinate)
        if let (Some(tc), Some(cc)) = (thrower_coord, catcher_coord) {
            if tc.is_adjacent(cc) {
                // Java: fieldModel.setBallCoordinate(catcherCoordinate)
                game.field_model.ball_coordinate = Some(cc);
                let from_id = game.acting_player.player_id.clone().unwrap_or_default();
                outcome = outcome
                    .with_event(GameEvent::HandOver { from_id, to_id: self.catcher_id.clone().unwrap_or_default() })
                    .publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchHandOff));
            }
        }

        // Java: boolean allowMoveAfterHandOff = thrower.hasSkillProperty(canMoveAfterHandOff)
        let allow_move_after_hand_off = game.acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::CAN_MOVE_AFTER_HAND_OFF))
            .unwrap_or(false);

        // Java: if (!allowMoveAfterHandOff) publishParameter(END_PLAYER_ACTION, true)
        if !allow_move_after_hand_off {
            outcome = outcome.publish(StepParameter::EndPlayerAction(true));
        }

        // Java: getResult().setNextAction(NEXT_STEP)
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn no_catcher_still_returns_next_step() {
        let mut game = make_game();
        let mut step = StepHandOver::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn publishes_end_player_action_when_no_move_skill() {
        let mut game = make_game();
        let mut step = StepHandOver::new();
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let epa = out.published.iter().find(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(epa.is_some(), "expected EndPlayerAction(true)");
    }

    #[test]
    fn adjacent_catcher_publishes_catch_hand_off_mode() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(6, 5)); // adjacent
        let mut step = StepHandOver::new();
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchHandOff))
        });
        assert!(mode.is_some(), "expected CatchScatterThrowInMode::CatchHandOff");
    }

    #[test]
    fn non_adjacent_catcher_does_not_publish_hand_off_mode() {
        let mut game = make_game();
        game.thrower_id = Some("t1".into());
        game.field_model.set_player_coordinate("t1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("c1", FieldCoordinate::new(10, 5)); // not adjacent
        let mut step = StepHandOver::new();
        step.catcher_id = Some("c1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        let mode = out.published.iter().find(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(_))
        });
        assert!(mode.is_none(), "should not publish mode for non-adjacent catcher");
    }

    #[test]
    fn sets_ball_moving_and_clears_pass_coordinate() {
        let mut game = make_game();
        game.pass_coordinate = Some(FieldCoordinate::new(7, 7));
        let mut step = StepHandOver::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
        assert!(game.pass_coordinate.is_none());
    }
}
