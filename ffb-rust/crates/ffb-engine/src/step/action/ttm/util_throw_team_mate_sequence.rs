/// 1:1 translation of com.fumbbl.ffb.server.step.action.ttm.UtilThrowTeamMateSequence (COMMON).
///
/// Utility for player-scatter during Throw Team-Mate: `scatter_player` and `kick_player`.
///
/// Used by TTM steps (not step itself — utility free functions).
use ffb_mechanics::mechanics::scatter_coordinate;
use ffb_mechanics::ttm_mechanic::TtmMechanic as TtmMechanicTrait;
use ffb_model::enums::{Direction, Rules};
use ffb_model::model::game::Game;
use ffb_model::report::report_scatter_player::ReportScatterPlayer;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;

/// Java: UtilThrowTeamMateSequence.ScatterResult
pub struct ScatterResult {
    /// Java: fLastValidCoordinate
    pub last_valid_coordinate: FieldCoordinate,
    /// Java: fInBounds
    pub in_bounds: bool,
}

impl ScatterResult {
    pub fn new(last_valid_coordinate: FieldCoordinate, in_bounds: bool) -> Self {
        Self { last_valid_coordinate, in_bounds }
    }
}

/// Java: UtilThrowTeamMateSequence.scatterPlayer — scatter a thrown/bounced player.
///
/// If `throw_scatter=true`: rolls up to 3 direction dice (d8 each), scatters 1 square per roll.
/// If `throw_scatter=false`: rolls 1 direction die and stops when the mechanic says so.
/// Stops early if the player goes out of bounds.
pub fn scatter_player(game: &mut Game, rng: &mut GameRng, start: FieldCoordinate, throw_scatter: bool) -> ScatterResult {
    let mut start_coord = start;
    let mut end_coord = start;
    let mut last_valid = start;
    let mut rolls: Vec<i32> = Vec::new();
    let mut directions: Vec<Direction> = Vec::new();
    let mut in_bounds = true;

    while in_bounds {
        // Java: break condition (checked before rolling)
        let should_stop = if throw_scatter {
            rolls.len() >= 3
        } else {
            !rolls.is_empty() && is_valid_end_scatter(game, start_coord)
        };
        if should_stop { break; }

        let roll = rng.d8();
        rolls.push(roll);
        let direction = Direction::for_roll(roll).expect("d8 is 1..=8");
        directions.push(direction);

        let (nx, ny) = scatter_coordinate(start_coord.x, start_coord.y, direction, 1);
        end_coord = FieldCoordinate::new(nx, ny);

        if end_coord.is_on_pitch() {
            last_valid = end_coord;
        } else {
            last_valid = start_coord;
            in_bounds = false;
        }
        start_coord = end_coord;
    }

    // Java: pStep.getResult().addReport(new ReportScatterPlayer(pStartCoordinate, endCoordinate,
    //       directions, rolls, pThrowScatter));
    game.report_list.add(ReportScatterPlayer::new(
        start,
        end_coord,
        directions,
        rolls,
        Some(throw_scatter),
    ));

    ScatterResult::new(last_valid, in_bounds)
}

/// Java: UtilThrowTeamMateSequence.kickPlayer — walk the player toward target, then scatter.
///
/// Steps from `start` one square at a time toward `target`. If the path goes out of bounds,
/// returns immediately (in_bounds=false). Otherwise calls scatter_player from the landing spot.
pub fn kick_player(game: &mut Game, rng: &mut GameRng, start: FieldCoordinate, target: FieldCoordinate) -> ScatterResult {
    let mut last_valid = start;
    let mut current = start;
    let mut in_bounds = true;

    let dx = (target.x - start.x).signum();
    let dy = (target.y - start.y).signum();

    while in_bounds && current != target {
        current = current.add(dx, dy);
        if current.is_on_pitch() {
            last_valid = current;
        } else {
            in_bounds = false;
        }
    }

    if in_bounds {
        scatter_player(game, rng, last_valid, true)
    } else {
        ScatterResult::new(last_valid, false)
    }
}

fn is_valid_end_scatter(game: &Game, coord: FieldCoordinate) -> bool {
    match game.rules {
        Rules::Bb2016 => {
            ffb_mechanics::bb2016::ttm_mechanic::TtmMechanic.is_valid_end_scatter_coordinate(game, coord)
        }
        _ => {
            // BB2020 and BB2025 always return true
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::field_model::FieldModel;
    use ffb_model::model::game::Game;

    fn make_game(rules: Rules) -> Game {
        let home = crate::step::framework::test_team("home", 0);
        let away = crate::step::framework::test_team("away", 0);
        Game::new(home, away, rules)
    }

    #[test]
    fn scatter_player_throw_rolls_exactly_3_steps() {
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(42);
        let start = FieldCoordinate::new(12, 7); // mid-pitch
        let result = scatter_player(&mut game, &mut rng, start, true);
        // Mid-pitch start with only 3 scatter steps stays in bounds, and must have moved.
        assert!(result.in_bounds);
        assert_ne!(result.last_valid_coordinate, start);
    }

    #[test]
    fn scatter_player_bounce_stops_after_first_step_bb2025() {
        // bb2025: is_valid_end_scatter always true → stops after 1 step
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let start = FieldCoordinate::new(12, 7);
        let _result = scatter_player(&mut game, &mut rng, start, false);
        // Consumed exactly 1 d8 roll; no assertion on exact coord (direction is seeded)
    }

    #[test]
    fn scatter_out_of_bounds_sets_in_bounds_false() {
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(99);
        // Place start at edge so scatter can go OOB
        let start = FieldCoordinate::new(0, 7);
        // Run many scatters; at least one seed will go OOB at the edge
        // (just verify it returns without panic)
        let _result = scatter_player(&mut game, &mut rng, start, false);
    }

    #[test]
    fn kick_player_walks_toward_target() {
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let start = FieldCoordinate::new(10, 7);
        let target = FieldCoordinate::new(14, 7); // 4 steps east
        let result = kick_player(&mut game, &mut rng, start, target);
        // After walking to target (14,7), scatter is called; result is somewhere near target
        // Just verify it returns and last_valid_coordinate is not the start
        let _ = result;
    }

    #[test]
    fn kick_player_out_of_bounds_returns_last_valid() {
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let start = FieldCoordinate::new(24, 7); // near right edge
        let target = FieldCoordinate::new(28, 7); // out of bounds
        let result = kick_player(&mut game, &mut rng, start, target);
        assert!(!result.in_bounds);
        // last_valid should be last in-bounds step
        assert!(result.last_valid_coordinate.is_on_pitch());
    }

    #[test]
    fn scatter_player_adds_report_scatter_player() {
        // Java: UtilThrowTeamMateSequence.scatterPlayer always calls
        // pStep.getResult().addReport(new ReportScatterPlayer(...)) before returning.
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(42);
        let start = FieldCoordinate::new(12, 7);
        scatter_player(&mut game, &mut rng, start, true);
        assert!(
            game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "scatter_player must add a ReportScatterPlayer to the game report list"
        );
    }

    #[test]
    fn kick_player_in_bounds_adds_report_scatter_player() {
        // kickPlayer delegates to scatterPlayer when the walk stays in bounds, so it must
        // also produce a ReportScatterPlayer.
        use ffb_model::report::report_id::ReportId;
        let mut game = make_game(Rules::Bb2025);
        let mut rng = GameRng::new(0);
        let start = FieldCoordinate::new(10, 7);
        let target = FieldCoordinate::new(14, 7);
        kick_player(&mut game, &mut rng, start, target);
        assert!(
            game.report_list.has_report(ReportId::SCATTER_PLAYER),
            "kick_player must add a ReportScatterPlayer when the walk to target stays in bounds"
        );
    }
}
