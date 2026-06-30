// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerSetup.
//
// Translated methods:
//   setup_player(game, player_id, coordinate) — pure game-model player placement
//
// Skipped (touch DB / WebSocket / TeamSetup persistence):
//   loadTeamSetup, saveTeamSetup, deleteTeamSetup

use ffb_model::enums::{PlayerState, PS_RESERVE, PS_STANDING};
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;

pub struct UtilServerSetup;

impl UtilServerSetup {
    /// Java: UtilServerSetup.setupPlayer(GameState, String playerId, FieldCoordinate)
    ///
    /// Moves `player_id` to `coordinate` on the field during the setup phase.
    ///
    /// Rules applied (1:1 from Java):
    /// * Player must exist and must belong to the currently-playing team.
    /// * Away team coordinates are mirrored via `transform()`.
    /// * If the target square already has a player the move is rejected.
    /// * Box coordinates → player goes to RESERVE.
    /// * QUICK_SNAP turn + coordinate changed → STANDING (active = false).
    /// * Otherwise → STANDING (active = true).
    pub fn setup_player(
        game: &mut Game,
        player_id: &str,
        coordinate: FieldCoordinate,
    ) {
        if player_id.is_empty() {
            return;
        }

        // Player must exist.
        if game.player(player_id).is_none() {
            return;
        }

        // Player must belong to the currently-playing team.
        let home_team_has_player = game.team_home.has_player(player_id);
        if home_team_has_player != game.home_playing {
            return;
        }

        // Away team uses the mirrored coordinate.
        let mapped_coordinate = if home_team_has_player {
            coordinate
        } else {
            coordinate.transform()
        };

        // Reject if another player already stands at that square.
        if game.field_model.player_at(mapped_coordinate).is_some() {
            return;
        }

        // Determine the new player state.
        let old_coordinate = game.field_model.player_coordinate(player_id);
        let old_state = game
            .field_model
            .player_state(player_id)
            .unwrap_or(PlayerState(PS_STANDING));

        let new_state = if mapped_coordinate.is_box_coordinate() {
            old_state.change_base(PS_RESERVE)
        } else if game.turn_mode == TurnMode::QuickSnap
            && old_coordinate != Some(mapped_coordinate)
        {
            old_state.change_base(PS_STANDING).change_active(false)
        } else {
            old_state.change_base(PS_STANDING).change_active(true)
        };

        game.field_model.set_player_state(player_id, new_state);
        game.field_model.set_player_coordinate(player_id, mapped_coordinate);
    }
}

impl Default for UtilServerSetup {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PlayerType, PlayerGender, PS_RESERVE, PS_STANDING};
    use ffb_model::enums::{Rules, TurnMode};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

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

    fn make_team(id: &str, player_ids: &[&str]) -> Team {
        let players = player_ids
            .iter()
            .map(|pid| make_test_player(pid))
            .collect();
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

    fn game_with_home_player(player_id: &str) -> Game {
        let home = make_team("home", &[player_id]);
        let away = make_team("away", &[]);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn setup_player_unknown_id_is_no_op() {
        let mut game = game_with_home_player("p1");
        let coord = FieldCoordinate::new(5, 5);
        UtilServerSetup::setup_player(&mut game, "unknown", coord);
        assert!(game.field_model.player_coordinate("unknown").is_none());
    }

    #[test]
    fn setup_player_wrong_team_is_rejected() {
        let home = make_team("home", &[]);
        let away = make_team("away", &["p_away"]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        // home_playing is true; p_away belongs to away team → rejected.
        let coord = FieldCoordinate::new(5, 5);
        UtilServerSetup::setup_player(&mut game, "p_away", coord);
        assert!(game.field_model.player_coordinate("p_away").is_none());
    }

    #[test]
    fn setup_player_places_home_player_at_coordinate() {
        let mut game = game_with_home_player("p1");
        let coord = FieldCoordinate::new(10, 7);
        UtilServerSetup::setup_player(&mut game, "p1", coord);
        assert_eq!(game.field_model.player_coordinate("p1"), Some(coord));
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
        assert!(state.is_active());
    }

    #[test]
    fn setup_player_box_coordinate_sets_reserve() {
        let mut game = game_with_home_player("p1");
        // Use a known box coordinate (RSV_HOME_X = -1).
        let box_coord = FieldCoordinate::new(-1, 5);
        assert!(box_coord.is_box_coordinate());
        UtilServerSetup::setup_player(&mut game, "p1", box_coord);
        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_RESERVE);
    }

    #[test]
    fn setup_player_occupied_square_is_rejected() {
        let home = make_team("home", &["p1", "p2"]);
        let away = make_team("away", &[]);
        let mut game = Game::new(home, away, Rules::Bb2025);
        let coord = FieldCoordinate::new(10, 7);
        // Place p1 at coord first.
        game.field_model.set_player_coordinate("p1", coord);
        // Attempt to place p2 at the same coordinate.
        UtilServerSetup::setup_player(&mut game, "p2", coord);
        // p2 should not be at coord; p1 should remain there.
        assert!(game.field_model.player_coordinate("p2").is_none());
        assert_eq!(game.field_model.player_coordinate("p1"), Some(coord));
    }

    #[test]
    fn setup_player_quick_snap_changed_coord_active_false() {
        let mut game = game_with_home_player("p1");
        game.turn_mode = TurnMode::QuickSnap;
        let old_coord = FieldCoordinate::new(5, 5);
        let new_coord = FieldCoordinate::new(6, 5);
        game.field_model.set_player_coordinate("p1", old_coord);

        UtilServerSetup::setup_player(&mut game, "p1", new_coord);

        let state = game.field_model.player_state("p1").unwrap();
        assert_eq!(state.base(), PS_STANDING);
        assert!(!state.is_active(), "quick-snap move should be active=false");
    }

    #[test]
    fn setup_player_empty_id_is_no_op() {
        let mut game = game_with_home_player("p1");
        let coord = FieldCoordinate::new(5, 5);
        UtilServerSetup::setup_player(&mut game, "", coord);
        // Nothing should have changed.
        assert!(game.field_model.player_coordinate("").is_none());
    }
}
