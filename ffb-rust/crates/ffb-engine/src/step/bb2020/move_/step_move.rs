use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2020.move.StepMove.
///
/// BB2020 logic is identical to BB2025.
/// Physically moves the acting player one square: updates the field model,
/// increments currentMove (×2 for jumping), optionally moves the ball if carried,
/// publishes PLAYER_ENTERING_SQUARE.
///
/// DEFERRED(trackNumber): TrackNumber animation not ported — no Rust FieldModel.track_numbers field.
pub struct StepMove {
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: fMoveStackSize — size of remaining move stack (0 = last step)
    pub move_stack_size: i32,
}

impl StepMove {
    pub fn new() -> Self {
        Self { coordinate_from: None, coordinate_to: None, move_stack_size: 0 }
    }
}

impl Default for StepMove {
    fn default() -> Self { Self::new() }
}

impl Step for StepMove {
    fn id(&self) -> StepId { StepId::Move }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::MoveStack(v) => { self.move_stack_size = v.len() as i32; true }
            _ => false,
        }
    }
}

impl StepMove {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let Some(ref attacker_id) = game.acting_player.player_id.clone() else {
            return StepOutcome::next();
        };

        let is_pinned = game.field_model.player_state(attacker_id)
            .map(|s| s.is_pinned())
            .unwrap_or(false);
        if is_pinned {
            return StepOutcome::next();
        }

        let Some(to) = self.coordinate_to else {
            return StepOutcome::next();
        };

        let jump_inc = if game.acting_player.jumping { 2 } else { 1 };
        game.acting_player.current_move += jump_inc;

        let old_pos = game.field_model.player_coordinate(attacker_id);
        let ball_position_updated = if !game.field_model.ball_moving {
            if let (Some(old), Some(ball)) = (old_pos, game.field_model.ball_coordinate) {
                if old == ball {
                    game.field_model.ball_coordinate = Some(to);
                    true
                } else { false }
            } else { false }
        } else { false };
        game.field_model.set_player_coordinate(attacker_id, to);

        // Java: if (ballPositionUpdated) { playerResult.setRushing(rushing + deltaX) }
        if ball_position_updated {
            let from_x = self.coordinate_from.map(|c| c.x).unwrap_or(to.x);
            let delta_x = if game.home_playing { to.x - from_x } else { from_x - to.x };
            let is_home = game.team_home.player(attacker_id).is_some();
            let pr = if is_home {
                game.game_result.home.player_results.entry(attacker_id.clone()).or_default()
            } else {
                game.game_result.away.player_results.entry(attacker_id.clone()).or_default()
            };
            pr.rushing += delta_x;
        }

        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);

        StepOutcome::next()
            .publish(StepParameter::PlayerEnteringSquare(attacker_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepParameter;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn no_acting_player_returns_next_step() {
        let mut game = make_game();
        let mut step = StepMove::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
    }

    #[test]
    fn moves_player_to_coordinate_to() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.field_model.set_player_coordinate("p1", from);
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(to));
    }

    #[test]
    fn publishes_player_entering_square() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        let out = step.start(&mut game, &mut GameRng::new(0));
        let has = out.published.iter().any(|p| matches!(p, StepParameter::PlayerEnteringSquare(id) if id == "p1"));
        assert!(has, "PlayerEnteringSquare must be published");
    }

    #[test]
    fn increments_current_move_by_one_for_non_jumping() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.current_move = 2;
        game.acting_player.jumping = false;
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 3);
    }
}
