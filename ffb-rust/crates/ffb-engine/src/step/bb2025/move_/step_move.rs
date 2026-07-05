use ffb_model::types::FieldCoordinate;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::util::UtilServerPlayerMove;
use crate::util::server_util_block::ServerUtilBlock;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.move.StepMove.
///
/// Physically moves the acting player one square: updates the field model,
/// increments currentMove (×2 for jumping), optionally moves the ball if carried,
/// publishes PLAYER_ENTERING_SQUARE.
///
/// Expects: COORDINATE_FROM, COORDINATE_TO, MOVE_STACK (size only) set by preceding step.
///
/// client-only: field_model.add(trackNumber) — TrackNumber animation is client-side display only.
/// client-only: SoundId assignment for DODGE/STEP — sound playback is client-only.
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
            // No acting player → NEXT_STEP (matches Java: skips body, falls to setNextAction)
            return StepOutcome::next();
        };

        // Java: playerState.isPinned() → skip body entirely (chomped or rooted)
        let is_pinned = game.field_model.player_state(attacker_id)
            .map(|s| s.is_pinned())
            .unwrap_or(false);
        if is_pinned {
            return StepOutcome::next();
        }

        let Some(to) = self.coordinate_to else {
            return StepOutcome::next();
        };

        // Java: actingPlayer.setCurrentMove(currentMove + (jumping ? 2 : 1))
        let jump_inc = if game.acting_player.jumping { 2 } else { 1 };
        game.acting_player.current_move += jump_inc;

        // Java: game.getFieldModel().add(trackNumber) — TODO: TrackNumber not ported
        // Java: updatePlayerAndBallPosition(actingPlayer.getPlayer(), fCoordinateTo)
        // If player is carrying ball (not moving), move ball to new square first.
        let old_pos = game.field_model.player_coordinate(attacker_id);
        let ball_position_updated = !game.field_model.ball_moving
            && old_pos.is_some()
            && old_pos == game.field_model.ball_coordinate;
        if ball_position_updated {
            game.field_model.ball_coordinate = Some(to);
        }
        game.field_model.set_player_coordinate(attacker_id, to);
        if ball_position_updated {
            if let Some(old) = old_pos {
                let delta_x = if game.home_playing { to.x - old.x } else { old.x - to.x };
                let is_home = game.team_home.has_player(attacker_id);
                let team_result = if is_home { &mut game.game_result.home } else { &mut game.game_result.away };
                let pr = team_result.player_results.entry(attacker_id.to_string()).or_default();
                pr.rushing += delta_x;
            }
        }
        game.acting_player.goes_for_it = UtilPlayer::is_next_move_going_for_it(game);
        if self.move_stack_size == 0 {
            UtilServerPlayerMove::update_move_squares(game, false);
        }
        ServerUtilBlock::update_dice_decorations(game);
        // client-only: SoundId DODGE/STEP — sound playback is client-only

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
        Game::new(home, away, Rules::Bb2025)
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

    #[test]
    fn increments_current_move_by_two_for_jumping() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.current_move = 2;
        game.acting_player.jumping = true;
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(7, 5));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.acting_player.current_move, 4);
    }

    #[test]
    fn rooted_player_does_not_move() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", from);
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING).change_rooted(true));
        let mut step = StepMove::new();
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.player_coordinate("p1"), Some(from));
    }

    #[test]
    fn ball_moves_with_carrier() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("p1", from);
        game.field_model.ball_coordinate = Some(from);
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.field_model.ball_coordinate, Some(to));
    }

    #[test]
    fn no_coordinate_to_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut step = StepMove::new();
        // No coordinate_to set
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, crate::step::framework::StepAction::NextStep);
    }

    #[test]
    fn rushing_stat_updated_when_ball_moves_with_player_home_playing() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        });
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("p1", from);
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
    fn rushing_stat_not_updated_when_not_carrying_ball() {
        let mut game = make_game();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        game.acting_player.player_id = Some("p1".into());
        game.home_playing = true;
        game.field_model.set_player_coordinate("p1", from);
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 10)); // ball elsewhere
        game.field_model.ball_moving = false;
        let mut step = StepMove::new();
        step.coordinate_from = Some(from);
        step.coordinate_to = Some(to);
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.game_result.home.player_results.get("p1").map(|pr| pr.rushing).unwrap_or(0) == 0);
    }
}
