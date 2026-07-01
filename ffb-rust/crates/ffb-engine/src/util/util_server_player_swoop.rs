// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerPlayerSwoop.
//
// Translated methods:
//   update_swoop_squares(game, player_id) — computes cardinal swoop squares
//   add_swoop_square(game, coord)         — private helper, inserts into move_squares
//
// No methods skipped — the Java only touches FieldModel (no DB/WebSocket).
//
// NOTE: the property check (ttmScattersInSingleDirection) requires
// `Player::has_skill_property` which is not yet in this version of ffb-model.
// The check always returns false (no swoop squares are generated).
// Correct once has_skill_property / NamedProperties are added.

use ffb_model::model::game::Game;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};

pub struct UtilServerPlayerSwoop;

impl UtilServerPlayerSwoop {
    /// Java: UtilServerPlayerSwoop.updateSwoopSquares(GameState, Player<?> swoopingPlayer)
    ///
    /// Computes the squares a swooping player can land in and stores them in
    /// `game.field_model.move_squares`.
    ///
    /// Only players with the `ttmScattersInSingleDirection` property receive
    /// swoop squares (four cardinal directions).  All others receive no move
    /// squares.
    ///
    /// NOTE: property check is stubbed as `false` until `has_skill_property` is
    /// available in ffb-model.
    pub fn update_swoop_squares(game: &mut Game, player_id: &str) {
        game.field_model.move_squares.clear();

        let player_coord = game.field_model.player_coordinate(player_id);
        let player_coord = match player_coord {
            Some(c) if FieldCoordinateBounds::FIELD.is_in_bounds(c) => c,
            _ => return,
        };

        // TODO: replace `false` with:
        //   game.player(player_id)
        //       .map(|p| p.has_skill_property(NamedProperties::TTM_SCATTERS_IN_SINGLE_DIRECTION))
        //       .unwrap_or(false)
        // once has_skill_property is available.
        let has_property: bool = false;
        let _ = player_coord; // used below once has_property becomes meaningful

        if !has_property {
            return;
        }

        // Four cardinal directions: left, right, up, down.
        let candidates = [
            player_coord.add(-1, 0),
            player_coord.add(1, 0),
            player_coord.add(0, -1),
            player_coord.add(0, 1),
        ];

        for coord in candidates {
            if FieldCoordinateBounds::FIELD.is_in_bounds(coord) {
                Self::add_swoop_square(game, coord);
            }
        }
    }

    /// Java: private static void addSwoopSquare(GameState, FieldCoordinate)
    ///
    /// Inserts `coord` into `game.field_model.move_squares`.
    pub fn add_swoop_square(game: &mut Game, coord: FieldCoordinate) {
        game.field_model.add_move_square(MoveSquare::new(coord, 0, 0));
    }
}

impl Default for UtilServerPlayerSwoop {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PlayerType, PlayerGender};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::{FieldCoordinate, MoveSquare};

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
        }
    }

    fn make_test_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 0,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 3,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        }
    }

    fn game_with_player_at(player_id: &str, coord: FieldCoordinate) -> Game {
        let p = make_test_player(player_id);
        let home = make_team("home", vec![p]);
        let away = make_team("away", vec![]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.field_model.set_player_coordinate(player_id, coord);
        game
    }

    #[test]
    fn update_swoop_squares_clears_existing_move_squares() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(10, 7));
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(0, 0), 0, 0));
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(1, 1), 0, 0));
        UtilServerPlayerSwoop::update_swoop_squares(&mut game, "p1");
        // Property not available → clears squares, adds none.
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_swoop_squares_no_property_results_in_empty_squares() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(10, 7));
        UtilServerPlayerSwoop::update_swoop_squares(&mut game, "p1");
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_swoop_squares_unknown_player_no_panic() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(10, 7));
        UtilServerPlayerSwoop::update_swoop_squares(&mut game, "unknown");
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_swoop_squares_out_of_bounds_player_no_panic() {
        // Box coordinate — is_in_bounds returns false.
        let mut game = game_with_player_at("p1", FieldCoordinate::new(-1, 5));
        UtilServerPlayerSwoop::update_swoop_squares(&mut game, "p1");
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_swoop_squares_edge_position_no_panic() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(0, 0));
        UtilServerPlayerSwoop::update_swoop_squares(&mut game, "p1");
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn add_swoop_square_helper_inserts_coordinate() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(5, 5));
        let coord = FieldCoordinate::new(6, 5);
        UtilServerPlayerSwoop::add_swoop_square(&mut game, coord);
        assert!(game.field_model.get_move_square(coord).is_some());
    }

    #[test]
    fn add_swoop_square_multiple_distinct_coordinates() {
        let mut game = game_with_player_at("p1", FieldCoordinate::new(5, 5));
        let c1 = FieldCoordinate::new(4, 5);
        let c2 = FieldCoordinate::new(6, 5);
        UtilServerPlayerSwoop::add_swoop_square(&mut game, c1);
        UtilServerPlayerSwoop::add_swoop_square(&mut game, c2);
        assert_eq!(game.field_model.move_squares.len(), 2);
        assert!(game.field_model.get_move_square(c1).is_some());
        assert!(game.field_model.get_move_square(c2).is_some());
    }
}
