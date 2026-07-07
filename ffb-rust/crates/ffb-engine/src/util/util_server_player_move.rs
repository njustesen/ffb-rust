/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerPlayerMove.
///
/// Public methods:
///   - isValidMove (two overloads: ClientCommandMove / ClientCommandBlitzMove)
///   - updateMoveSquares(GameState, boolean jumping)
///   - fetchMoveStack (two overloads)
///   - fetchFromSquare (two overloads)
///
/// The Java class depends on ClientCommand types, DiceInterpreter, and factory
/// mechanics (JumpMechanic, AgilityMechanic, etc.). This Rust port implements
/// the core coordinate-transformation logic and the move-square scan, with
/// TODO stubs for the mechanics-dependent parts.
use ffb_model::model::game::Game;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};

pub struct UtilServerPlayerMove;

impl UtilServerPlayerMove {
    /// Java: UtilServerPlayerMove.fetchMoveStack(FieldCoordinate[], boolean homeCommand)
    ///
    /// Transforms coordinates from the away perspective to the home perspective
    /// when homeCommand is false.
    pub fn fetch_move_stack(coordinates_to: &[FieldCoordinate], home_command: bool) -> Vec<FieldCoordinate> {
        if coordinates_to.is_empty() {
            return vec![];
        }
        if home_command {
            coordinates_to.to_vec()
        } else {
            coordinates_to.iter().map(|c| c.transform()).collect()
        }
    }

    /// Java: UtilServerPlayerMove.fetchFromSquare(FieldCoordinate, boolean homeCommand)
    ///
    /// Transforms the from-coordinate from the away perspective when homeCommand is false.
    pub fn fetch_from_square(from: FieldCoordinate, home_command: bool) -> FieldCoordinate {
        if home_command { from } else { from.transform() }
    }

    /// Java: UtilServerPlayerMove.isValidMove(GameState, coordinateFrom, homeCommand)
    ///
    /// Checks that the acting player is at coordinateFrom. Drops the command if not
    /// (command out-of-sync). In this headless port we simply check the field model.
    pub fn is_valid_move(game: &Game, coordinate_from: FieldCoordinate, _home_command: bool) -> bool {
        let acting_id = match &game.acting_player.player_id {
            Some(id) => id,
            None => return false,
        };
        match game.field_model.player_coordinate(acting_id) {
            Some(player_coord) => player_coord == coordinate_from,
            None => false,
        }
    }

    /// Java: UtilServerPlayerMove.updateMoveSquares(GameState, boolean jumping)
    ///
    /// Recomputes which squares the acting player can legally move to and stores
    /// them in FieldModel.move_squares.
    ///
    /// The full implementation requires JumpMechanic, AgilityMechanic, DiceInterpreter,
    /// TurnMode checks (PassBlock/KickoffReturn), UtilPlayer::isNextMovePossible, etc.
    /// This port handles the basic non-jumping, non-special-mode scan.
    pub fn update_move_squares(game: &mut Game, jumping: bool) {
        game.field_model.move_squares.clear();

        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return,
        };

        let player_coord = match game.field_model.player_coordinate(&acting_id) {
            Some(c) => c,
            None => return,
        };

        let action_is_moving = game.acting_player.player_action
            .map(|a| a.is_moving())
            .unwrap_or(false);

        if !action_is_moving { return; }
        if !FieldCoordinateBounds::FIELD.is_in_bounds(player_coord) { return; }

        // Java: steps=2 for jump, 1 otherwise. For jumping we check 2-step Chebyshev range.
        let adjacent: Vec<FieldCoordinate> = if jumping {
            let mut coords = Vec::new();
            for dx in -2i32..=2 {
                for dy in -2i32..=2 {
                    if dx == 0 && dy == 0 { continue; }
                    let c = player_coord.add(dx, dy);
                    if FieldCoordinateBounds::FIELD.is_in_bounds(c) { coords.push(c); }
                }
            }
            coords
        } else {
            game.field_model.adjacent_on_pitch(player_coord)
        };

        for coord in adjacent {
            if game.field_model.player_at(coord).is_none() {
                game.field_model.add_move_square(MoveSquare::new(coord, 0, 0));
            }
        }
    }
}

impl Default for UtilServerPlayerMove {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{Rules, PlayerAction, PlayerState, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    // -- fetch_move_stack --

    #[test]
    fn fetch_move_stack_home_command_unchanged() {
        let coords = vec![FieldCoordinate::new(5, 7), FieldCoordinate::new(6, 7)];
        let result = UtilServerPlayerMove::fetch_move_stack(&coords, true);
        assert_eq!(result, coords);
    }

    #[test]
    fn fetch_move_stack_away_command_transforms() {
        let c = FieldCoordinate::new(10, 7);
        let result = UtilServerPlayerMove::fetch_move_stack(&[c], false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], c.transform());
    }

    #[test]
    fn fetch_move_stack_empty_input() {
        let result = UtilServerPlayerMove::fetch_move_stack(&[], true);
        assert!(result.is_empty());
    }

    // -- fetch_from_square --

    #[test]
    fn fetch_from_square_home_unchanged() {
        let c = FieldCoordinate::new(8, 5);
        assert_eq!(UtilServerPlayerMove::fetch_from_square(c, true), c);
    }

    #[test]
    fn fetch_from_square_away_transforms() {
        let c = FieldCoordinate::new(10, 7);
        assert_eq!(UtilServerPlayerMove::fetch_from_square(c, false), c.transform());
    }

    // -- is_valid_move --

    #[test]
    fn is_valid_move_true_when_player_at_coord() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        assert!(UtilServerPlayerMove::is_valid_move(&game, coord, true));
    }

    #[test]
    fn is_valid_move_false_when_player_elsewhere() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        let other = FieldCoordinate::new(6, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        assert!(!UtilServerPlayerMove::is_valid_move(&game, other, true));
    }

    #[test]
    fn is_valid_move_false_when_no_acting_player() {
        let game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        assert!(!UtilServerPlayerMove::is_valid_move(&game, coord, true));
    }

    // -- update_move_squares --

    #[test]
    fn update_move_squares_no_acting_player_is_noop() {
        let mut game = make_game();
        game.acting_player.player_id = None;
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_not_moving_action_is_noop() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        game.acting_player.player_action = Some(PlayerAction::Block);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_move_action_populates_squares() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(!game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_occupied_squares_excluded() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        let blocker_coord = FieldCoordinate::new(11, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        game.field_model.set_player_coordinate("blocker", blocker_coord);
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.get_move_square(blocker_coord).is_none());
    }

    #[test]
    fn update_move_squares_clears_previous_squares() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        game.acting_player.player_action = Some(PlayerAction::Move);
        // Add a stale square.
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(0, 0), 0, 0));
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        // Stale (0,0) is not adjacent to (10,7), so it should be gone.
        assert!(game.field_model.get_move_square(FieldCoordinate::new(0, 0)).is_none());
    }
}
