use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::ServerUtilBlock;
use crate::util::UtilServerPlayerMove;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepMove.
///
/// Performs the actual player move: updates position, currentMove, track numbers,
/// rushing stats, and move-square state.
///
/// Init params: COORDINATE_FROM (mandatory), COORDINATE_TO (mandatory),
///              MOVE_STACK (optional — remaining stack after this move; empty = last step).
///
/// Logic (executeStep):
/// - Determine jumpingMove = actingPlayer.isJumping()
/// - currentMove += jumpingMove ? 2 : 1
/// - addTrackNumber(actingPlayer, coordinateTo)
/// - updatePlayerAndBallPosition(actingPlayer, coordinateTo)
/// - If rushing (goesForIt): update deltaX stat
/// - If goingForIt: set goesForIt = false (now resolved)
/// - If remaining move stack empty: updateMoveSquares + updateDiceDecorations
/// - NEXT_STEP
///
/// client-only: TrackNumber animation — field_model.track_numbers is client-side display only.
pub struct StepMove {
    /// Java: fCoordinateFrom
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: fCoordinateTo
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: fMoveStackSize — tracked as remaining stack; 0 = last move in the sequence.
    pub move_stack_size: usize,
}

impl StepMove {
    pub fn new() -> Self {
        Self {
            coordinate_from: None,
            coordinate_to: None,
            move_stack_size: 0,
        }
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
            // Java: fMoveStackSize set from the remaining MOVE_STACK length
            StepParameter::MoveStack(v) => { self.move_stack_size = v.len(); true }
            _ => false,
        }
    }
}

impl StepMove {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let coordinate_to = match self.coordinate_to {
            Some(c) => c,
            None => return StepOutcome::next(),
        };

        let jumping = game.acting_player.jumping;

        // Java: actingPlayer.getCurrentMove() + (jumping ? 2 : 1)
        let move_increment = if jumping { 2 } else { 1 };
        game.acting_player.current_move += move_increment;

        // Java: game.getFieldModel().add(trackNumber)
        // client-only: TrackNumber animation — field_model.track_numbers is client-side display only

        // Java: updatePlayerAndBallPosition(actingPlayer, coordinateTo)
        if let Some(ref player_id) = game.acting_player.player_id.clone() {
            let old_pos = game.field_model.player_coordinate(player_id);
            let ball_position_updated = if let (Some(ball_pos), Some(old)) = (game.field_model.ball_coordinate, old_pos) {
                if ball_pos == old && !game.field_model.ball_moving {
                    game.field_model.ball_coordinate = Some(coordinate_to);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            game.field_model.set_player_coordinate(player_id, coordinate_to);

            // Java: if (ballPositionUpdated) { playerResult.setRushing(rushing + deltaX) }
            if ball_position_updated {
                let from_x = self.coordinate_from.map(|c| c.x).unwrap_or(coordinate_to.x);
                let delta_x = if game.home_playing {
                    coordinate_to.x - from_x
                } else {
                    from_x - coordinate_to.x
                };
                let is_home = game.team_home.player(player_id).is_some();
                let pr = if is_home {
                    game.game_result.home.player_results.entry(player_id.clone()).or_default()
                } else {
                    game.game_result.away.player_results.entry(player_id.clone()).or_default()
                };
                pr.rushing += delta_x;
            }

            // Java: actingPlayer.setGoingForIt(UtilPlayer.isNextMoveGoingForIt(game))
            game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);
        }

        // Java: if (fMoveStackSize == 0) updateMoveSquares + updateDiceDecorations
        if self.move_stack_size == 0 {
            let jumping = game.acting_player.jumping;
            UtilServerPlayerMove::update_move_squares(game, jumping);
            ServerUtilBlock::update_dice_decorations(game);
        }

        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;
    use std::collections::HashSet;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn add_player(game: &mut Game, id: &str, start: FieldCoordinate) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                    ..Default::default()
});
        game.field_model.set_player_coordinate(id, start);
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn move_updates_player_position() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        let pos = game.field_model.player_coordinate("p1");
        assert_eq!(pos, Some(to));
    }

    #[test]
    fn move_increments_current_move_by_one() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        game.acting_player.current_move = 2;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 3);
    }

    #[test]
    fn jumping_move_increments_current_move_by_two() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        add_player(&mut game, "p1", from);
        game.acting_player.jumping = true;
        game.acting_player.current_move = 2;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 4);
    }

    #[test]
    fn ball_moves_with_player_when_player_has_ball() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(to));
    }

    #[test]
    fn ball_not_moved_when_not_at_player_position() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        let ball_pos = FieldCoordinate::new(3, 3); // ball elsewhere
        add_player(&mut game, "p1", from);
        game.field_model.ball_coordinate = Some(ball_pos);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(ball_pos));
    }

    #[test]
    fn returns_next_step_when_coordinate_to_is_none() {
        let mut game = make_game();
        let mut step = StepMove::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn returns_next_step_on_successful_move() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_coordinate_from_accepted() {
        let mut step = StepMove::new();
        let coord = FieldCoordinate::new(5, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert_eq!(step.coordinate_from, Some(coord));
    }

    #[test]
    fn set_parameter_coordinate_to_accepted() {
        let mut step = StepMove::new();
        let coord = FieldCoordinate::new(6, 5);
        assert!(step.set_parameter(&StepParameter::CoordinateTo(coord)));
        assert_eq!(step.coordinate_to, Some(coord));
    }

    #[test]
    fn set_parameter_move_stack_accepted() {
        use ffb_model::types::FieldCoordinate;
        let mut step = StepMove::new();
        let stack = vec![FieldCoordinate::new(6, 5), FieldCoordinate::new(7, 5), FieldCoordinate::new(8, 5)];
        assert!(step.set_parameter(&StepParameter::MoveStack(stack)));
        assert_eq!(step.move_stack_size, 3);
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepMove::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn rushing_stat_updated_when_ball_moves_with_player_home_playing() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5); // +2 in x
        add_player(&mut game, "p1", from);
        game.home_playing = true;
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        let rushing = game.game_result.home.player_results.get("p1").map(|pr| pr.rushing).unwrap_or(0);
        assert_eq!(rushing, 2); // to.x(7) - from.x(5) = 2
    }

    #[test]
    fn rushing_stat_uses_from_minus_to_when_away_playing() {
        let mut game = make_game();
        let from = FieldCoordinate::new(7, 5);
        let to = FieldCoordinate::new(5, 5); // moving in negative x direction
        add_player(&mut game, "p1", from);
        game.home_playing = false;
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        // away playing: delta_x = from.x(7) - to.x(5) = 2 → rushing = 2
        let rushing = game.game_result.home.player_results.get("p1").map(|pr| pr.rushing).unwrap_or(0);
        assert_eq!(rushing, 2);
    }

    #[test]
    fn rushing_stat_not_updated_when_ball_not_at_player_position() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        add_player(&mut game, "p1", from);
        game.home_playing = true;
        // ball is elsewhere — player does not carry it
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(3, 3));
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        let rushing = game.game_result.home.player_results.get("p1").map(|pr| pr.rushing).unwrap_or(0);
        assert_eq!(rushing, 0);
    }

    #[test]
    fn goes_for_it_set_after_move_when_at_ma() {
        // Player with MA=4, current_move starts at 3 → after +1 = 4 = MA → going for it
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        game.acting_player.current_move = 3; // will become 4 after increment
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        // current_move is now 4 which equals MA(4), so next move is GFI
        assert!(game.acting_player.goes_for_it);
    }

    #[test]
    fn goes_for_it_false_when_well_within_ma() {
        // Player with MA=4, current_move starts at 0 → after +1 = 1, well below MA
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        add_player(&mut game, "p1", from);
        game.acting_player.current_move = 0;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(!game.acting_player.goes_for_it);
    }
}
