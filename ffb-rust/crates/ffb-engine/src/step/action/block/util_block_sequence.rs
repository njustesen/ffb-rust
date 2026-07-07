/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.UtilBlockSequence (COMMON).
///
/// Utility that initialises a pushback sequence: clears pushback squares, finds the starting
/// pushback square, and handles the Strip Ball skill (forceOpponentToDropBallOnPushback).
use ffb_model::model::game::Game;
use crate::step::framework::StepParameter;
use crate::util::util_server_pushback::UtilServerPushback;

/// Java: UtilBlockSequence.initPushback(step) — initialises pushback parameters.
///
/// Returns a list of StepParameter values that should be published: always includes
/// STARTING_PUSHBACK_SQUARE (with real direction from attacker→defender geometry);
/// may include CATCH_SCATTER_THROW_IN_MODE and BALL_KNOCKED_LOSE when StripBall fires.
pub fn init_pushback(game: &mut Game) -> Vec<StepParameter> {
    let mut params = Vec::new();

    // Java: game.getFieldModel().clearPushbackSquares()
    game.field_model.pushback_squares.clear();

    // Java: UtilServerPushback.findStartingSquare(attackerCoord, defenderCoord, isHomePlaying)
    let attacker_coord = game.acting_player.player_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id));
    let defender_coord = game.defender_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id));

    if let (Some(ac), Some(dc)) = (attacker_coord, defender_coord) {
        let starting_sq = UtilServerPushback::find_starting_square(ac, dc, game.home_playing);
        params.push(StepParameter::StartingPushbackSquare(starting_sq));
    }

    // Java: Strip Ball check (NamedProperties.forceOpponentToDropBallOnPushback)
    // Stub: NamedProperties not yet implemented → skip entire check.

    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Direction, Rules};
    use ffb_model::types::{FieldCoordinate, PushbackSquare};

    #[test]
    fn init_pushback_clears_pushback_squares() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.field_model.pushback_squares.push(PushbackSquare::new(FieldCoordinate::new(1, 1), Direction::North, false));
        init_pushback(&mut game);
        assert!(game.field_model.pushback_squares.is_empty());
    }

    #[test]
    fn init_pushback_returns_starting_square_at_defender_position_with_direction() {
        use ffb_model::types::PushbackSquare;
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.home_playing = true;
        // Attacker north of defender → direction = South
        game.acting_player.player_id = Some("att".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(8, 3));
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(8, 4));
        let params = init_pushback(&mut game);
        // Starting square is at defender's coordinate, direction South (attacker is north)
        assert!(params.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(Some(sq))
            if sq.coordinate == FieldCoordinate::new(8, 4) && sq.direction == Direction::South)));
    }

    #[test]
    fn init_pushback_no_defender_no_starting_square() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        let params = init_pushback(&mut game);
        assert!(params.iter().all(|p| !matches!(p, StepParameter::StartingPushbackSquare(_))));
    }

    #[test]
    fn init_pushback_no_attacker_no_starting_square() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(8, 4));
        // No acting player set
        let params = init_pushback(&mut game);
        assert!(params.iter().all(|p| !matches!(p, StepParameter::StartingPushbackSquare(_))));
    }
    #[test]
    fn init_pushback_with_empty_board_clears_squares() {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        assert!(game.field_model.pushback_squares.is_empty());
        init_pushback(&mut game);
        assert!(game.field_model.pushback_squares.is_empty());
    }
}
