/// 1:1 translation of com.fumbbl.ffb.server.step.action.block.UtilBlockSequence (COMMON).
///
/// Utility that initialises a pushback sequence: clears pushback squares, finds the starting
/// pushback square, and handles the Strip Ball skill (forceOpponentToDropBallOnPushback).
///
/// Stub: `find_starting_square` returns the defender coordinate (UtilServerPushback is not yet
/// translated).  Strip Ball (NamedProperties.forceOpponentToDropBallOnPushback) is not checked.
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::StepParameter;

/// Java: UtilBlockSequence.initPushback(step) — initialises pushback parameters.
///
/// Returns a list of StepParameter values that should be published: always includes
/// STARTING_PUSHBACK_SQUARE (defender coordinate stub); may include
/// CATCH_SCATTER_THROW_IN_MODE and BALL_KNOCKED_LOSE when StripBall fires.
pub fn init_pushback(game: &mut Game) -> Vec<StepParameter> {
    let mut params = Vec::new();

    // Java: game.getFieldModel().clearPushbackSquares()
    game.field_model.pushback_squares.clear();

    // Java: UtilServerPushback.findStartingSquare(attackerCoord, defenderCoord, isHomePlaying)
    // Stub: starting square = defender coordinate.
    let starting_sq: Option<FieldCoordinate> = game.defender_id.as_deref()
        .and_then(|id| game.field_model.player_coordinate(id));

    if let Some(sq) = starting_sq {
        params.push(StepParameter::StartingPushbackSquare(sq));
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
    fn init_pushback_returns_starting_square_at_defender_position() {
        let mut game = {
            let home = test_team("home", 0);
            let away = test_team("away", 0);
            Game::new(home, away, Rules::Bb2025)
        };
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(8, 4));
        let params = init_pushback(&mut game);
        assert!(params.iter().any(|p| matches!(p, StepParameter::StartingPushbackSquare(c) if *c == FieldCoordinate::new(8, 4))));
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
}
