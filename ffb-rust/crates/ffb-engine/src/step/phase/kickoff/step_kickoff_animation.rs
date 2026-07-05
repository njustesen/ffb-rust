/// 1:1 translation of com.fumbbl.ffb.server.step.phase.kickoff.StepKickoffAnimation.
///
/// Expects stepParameter KICKED_PLAYER_COORDINATE to be set by a preceding step.
/// Expects stepParameter TOUCHBACK to be set by a preceding step.
///
/// Sets stepParameter CATCH_SCATTER_THROW_IN_MODE for all steps on the stack.
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepId, StepOutcome, StepParameter};

pub struct StepKickoffAnimation {
    /// Java: fKickingPlayerCoordinate — coordinate of the kicker; resolved from home_playing if None.
    kicking_player_coordinate: Option<FieldCoordinate>,
    /// Java: fTouchback — whether a touchback was triggered.
    touchback: bool,
}

impl StepKickoffAnimation {
    pub fn new() -> Self {
        Self { kicking_player_coordinate: None, touchback: false }
    }
}

impl Default for StepKickoffAnimation {
    fn default() -> Self { Self::new() }
}

impl Step for StepKickoffAnimation {
    fn id(&self) -> StepId { StepId::KickoffAnimation }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::KickingPlayerCoordinate(coord) => {
                self.kicking_player_coordinate = Some(*coord);
                true
            }
            StepParameter::Touchback(v) => {
                self.touchback = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepKickoffAnimation {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: if (fKickingPlayerCoordinate == null) {
        //   if (game.isHomePlaying()) fKickingPlayerCoordinate = new FieldCoordinate(2, 8)
        //   else fKickingPlayerCoordinate = new FieldCoordinate(27, 8)
        let _kicking_coord = self.kicking_player_coordinate.unwrap_or_else(|| {
            if game.home_playing {
                FieldCoordinate::new(2, 8)
            } else {
                FieldCoordinate::new(27, 8)
            }
        });
        // Java: game.getFieldModel().setBallInPlay(true)
        game.field_model.ball_in_play = true;
        // Java: getResult().setAnimation(new Animation(AnimationType.KICK, ...))
        // client-only: animation reporting — no game state effect
        // Java: if (!fTouchback) publishParameter(CATCH_SCATTER_THROW_IN_MODE, CATCH_KICKOFF)
        let mut outcome = StepOutcome::next();
        if !self.touchback {
            outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                CatchScatterThrowInMode::CatchKickoff,
            ));
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn start_sets_ball_in_play() {
        let mut game = make_game();
        let mut step = StepKickoffAnimation::new();
        assert!(!game.field_model.ball_in_play);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_in_play);
    }

    #[test]
    fn without_touchback_publishes_catch_kickoff_mode() {
        let mut game = make_game();
        let mut step = StepKickoffAnimation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_mode = out.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::CatchKickoff))
        });
        assert!(has_mode, "should publish CatchScatterThrowInMode::CatchKickoff when not touchback");
    }

    #[test]
    fn with_touchback_does_not_publish_catch_kickoff_mode() {
        let mut game = make_game();
        let mut step = StepKickoffAnimation::new();
        step.set_parameter(&StepParameter::Touchback(true));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        let has_mode = out.published.iter().any(|p| {
            matches!(p, StepParameter::CatchScatterThrowInMode(_))
        });
        assert!(!has_mode, "should NOT publish CatchScatterThrowInMode when touchback");
    }

    #[test]
    fn set_parameter_kicking_player_coordinate() {
        let mut step = StepKickoffAnimation::new();
        assert!(step.set_parameter(&StepParameter::KickingPlayerCoordinate(FieldCoordinate::new(2, 8))));
        assert_eq!(step.kicking_player_coordinate, Some(FieldCoordinate::new(2, 8)));
    }

    #[test]
    fn set_parameter_touchback() {
        let mut step = StepKickoffAnimation::new();
        assert!(step.set_parameter(&StepParameter::Touchback(true)));
        assert!(step.touchback);
    }

    #[test]
    fn set_parameter_unrecognized_returns_false() {
        let mut step = StepKickoffAnimation::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn home_playing_default_coordinate_is_2_8() {
        let mut game = make_game();
        game.home_playing = true;
        let mut step = StepKickoffAnimation::new();
        // kicking_player_coordinate is None — step resolves to (2,8) internally
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn away_playing_default_coordinate_is_27_8() {
        let mut game = make_game();
        game.home_playing = false;
        let mut step = StepKickoffAnimation::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }
}
