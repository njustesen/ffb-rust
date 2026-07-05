/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerCatchScatterThrowIn.
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::team::Team;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::Direction;
use ffb_model::util::util_player::UtilPlayer;

pub struct UtilServerCatchScatterThrowIn;

impl UtilServerCatchScatterThrowIn {
    /// 1:1 translation of UtilServerCatchScatterThrowIn.findScatterCoordinate.
    ///
    /// Returns the coordinate obtained by stepping `dist` squares in direction
    /// `dir` from `start`.  Java does per-direction arithmetic; `FieldCoordinate::step`
    /// covers all 8 directions identically.
    pub fn find_scatter_coordinate(start: FieldCoordinate, dir: Direction, dist: i32) -> FieldCoordinate {
        start.step(dir, dist)
    }

    /// 1:1 translation of UtilServerCatchScatterThrowIn.findDivingCatchers.
    ///
    /// Returns all players on `team` who are adjacent to `coord`, have tackle-zones
    /// (i.e. are standing and not confused/hypnotized), and have the diving-catch
    /// skill property (`canAttemptCatchInAdjacentSquares`).  Sorted by player number
    /// (ascending) to match Java's `UtilPlayer.sortByPlayerNr`.
    pub fn find_diving_catchers<'a>(game: &'a Game, team: &'a Team, coord: FieldCoordinate) -> Vec<&'a Player> {
        let ids = UtilPlayer::find_adjacent_players_with_tacklezones(game, team, coord, false);
        let mut catchers: Vec<&Player> = ids.iter()
            .filter_map(|id| game.player(id))
            .filter(|p| p.has_skill_property(NamedProperties::CAN_ATTEMPT_CATCH_IN_ADJACENT_SQUARES))
            .collect();
        catchers.sort_by_key(|p| p.nr);
        catchers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Direction, Rules, PS_STANDING, PlayerState};
    use ffb_model::model::{Game, Team, Player};
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn make_player(id: &str, nr: i32, starting_skills: Vec<SkillWithValue>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills, extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn find_scatter_coordinate_north() {
        let start = FieldCoordinate::new(5, 5);
        let end = UtilServerCatchScatterThrowIn::find_scatter_coordinate(start, Direction::North, 1);
        assert_eq!(end, FieldCoordinate::new(5, 4));
    }

    #[test]
    fn find_scatter_coordinate_southeast() {
        let start = FieldCoordinate::new(5, 5);
        let end = UtilServerCatchScatterThrowIn::find_scatter_coordinate(start, Direction::Southeast, 1);
        assert_eq!(end, FieldCoordinate::new(6, 6));
    }

    #[test]
    fn find_diving_catchers_empty_field() {
        let game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        let catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(&game, &game.team_home, coord);
        assert!(catchers.is_empty());
    }

    #[test]
    fn find_diving_catchers_finds_skilled_player() {
        let mut game = make_game();
        let player = make_player("h1", 1, vec![SkillWithValue::new(SkillId::DivingCatch)]);
        let adj_coord = FieldCoordinate::new(5, 4); // north of (5,5)
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("h1", adj_coord);
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let coord = FieldCoordinate::new(5, 5);
        let catchers = UtilServerCatchScatterThrowIn::find_diving_catchers(&game, &game.team_home, coord);
        assert_eq!(catchers.len(), 1);
        assert_eq!(catchers[0].id, "h1");
    }
}
