/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbPlayerMarkersInsertParameterList.
use ffb_model::util::string_tool::is_provided;

use crate::db::i_db_update_parameter::IDbUpdateParameter;
use crate::db::i_db_update_parameter_list::IDbUpdateParameterList;
use crate::game_state::GameState;

use super::db_player_markers_insert_parameter::DbPlayerMarkersInsertParameter;

pub struct DbPlayerMarkersInsertParameterList {
    parameters: Vec<DbPlayerMarkersInsertParameter>,
}

impl DbPlayerMarkersInsertParameterList {
    pub fn new() -> Self {
        Self { parameters: Vec::new() }
    }

    pub fn add_parameter(&mut self, parameter: DbPlayerMarkersInsertParameter) {
        self.parameters.push(parameter);
    }

    pub fn get_parameters(&self) -> &[DbPlayerMarkersInsertParameter] {
        &self.parameters
    }

    /// Java: `initFrom(GameState pGameState, boolean loadAutoHome, boolean loadAutoAway)`.
    pub fn init_from(
        &mut self,
        game_state: Option<&GameState>,
        load_auto_home: bool,
        load_auto_away: bool,
    ) {
        let game_state = match game_state {
            Some(gs) => gs,
            None => return,
        };
        let game = match game_state.get_game() {
            Some(g) => g,
            None => return,
        };
        let team_home = &game.team_home;
        let team_away = &game.team_away;
        if !is_provided(Some(team_home.id.as_str())) || !is_provided(Some(team_away.id.as_str())) {
            return;
        }
        for player_marker in game.field_model.get_player_markers() {
            let player_id = match player_marker.get_player_id() {
                Some(id) => id,
                None => continue,
            };
            if !load_auto_home
                && team_home.has_player(player_id)
                && is_provided(player_marker.get_home_text())
            {
                self.add_parameter(DbPlayerMarkersInsertParameter::new(
                    team_home.id.clone(),
                    player_id.to_string(),
                    player_marker.get_home_text().unwrap().to_string(),
                ));
            }
            if !load_auto_away
                && team_away.has_player(player_id)
                && is_provided(player_marker.get_away_text())
            {
                self.add_parameter(DbPlayerMarkersInsertParameter::new(
                    team_away.id.clone(),
                    player_id.to_string(),
                    player_marker.get_away_text().unwrap().to_string(),
                ));
            }
        }
    }
}

impl Default for DbPlayerMarkersInsertParameterList {
    fn default() -> Self {
        Self::new()
    }
}

impl IDbUpdateParameterList for DbPlayerMarkersInsertParameterList {
    /// Java: `getParameters()` returns `DbPlayerMarkersInsertParameter[]` (a covariant
    /// override of the interface's return type). This translates the interface-level
    /// signature, boxing clones of the concrete parameters as trait objects.
    fn get_parameters(&self) -> Vec<Box<dyn IDbUpdateParameter>> {
        self.parameters
            .iter()
            .cloned()
            .map(|p| Box::new(p) as Box<dyn IDbUpdateParameter>)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_engine::step::driver::DriverGameState;
    use ffb_model::enums::Rules;
    use ffb_model::marking::player_marker::PlayerMarker;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn team(id: &str, player_ids: &[&str]) -> Team {
        Team {
            id: id.to_string(),
            name: "Team".into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
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
            players: player_ids
                .iter()
                .map(|id| Player { id: id.to_string(), ..Default::default() })
                .collect(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn game_state_with(home: Team, away: Team, markers: Vec<PlayerMarker>) -> GameState {
        let mut game = Game::new(home, away, Rules::Bb2020);
        game.field_model.player_markers = markers;
        let mut gs = GameState::new(1);
        gs.driver = Some(DriverGameState::from_game(game, 0));
        gs
    }

    #[test]
    fn construct() {
        let list = DbPlayerMarkersInsertParameterList::new();
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn init_from_none_game_state_is_noop() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        list.init_from(None, false, false);
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn init_from_not_started_game_state_is_noop() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        let gs = GameState::new(1);
        list.init_from(Some(&gs), false, false);
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn init_from_missing_team_ids_is_noop() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        let home = team("", &["p1"]);
        let away = team("t2", &["p2"]);
        let mut marker = PlayerMarker::with_player_id("p1");
        marker.set_home_text("hi");
        let gs = game_state_with(home, away, vec![marker]);
        list.init_from(Some(&gs), false, false);
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn init_from_adds_home_and_away_parameters() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        let home = team("t1", &["p1"]);
        let away = team("t2", &["p2"]);
        let mut marker_home = PlayerMarker::with_player_id("p1");
        marker_home.set_home_text("Home note");
        let mut marker_away = PlayerMarker::with_player_id("p2");
        marker_away.set_away_text("Away note");
        let gs = game_state_with(home, away, vec![marker_home, marker_away]);

        list.init_from(Some(&gs), false, false);

        let params = list.get_parameters();
        assert_eq!(params.len(), 2);
        assert!(params.iter().any(|p| p.get_team_id() == "t1" && p.get_player_id() == "p1" && p.get_text() == "Home note"));
        assert!(params.iter().any(|p| p.get_team_id() == "t2" && p.get_player_id() == "p2" && p.get_text() == "Away note"));
    }

    #[test]
    fn init_from_skips_empty_text_and_auto_flags() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        let home = team("t1", &["p1"]);
        let away = team("t2", &["p2"]);
        let mut marker_home = PlayerMarker::with_player_id("p1");
        marker_home.set_home_text("Home note");
        let gs = game_state_with(home, away, vec![marker_home]);

        // load_auto_home = true means we skip inserting home markers.
        list.init_from(Some(&gs), true, false);
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn add_parameter() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        list.add_parameter(DbPlayerMarkersInsertParameter::new(
            "t1".to_string(),
            "p1".to_string(),
            "text".to_string(),
        ));
        assert_eq!(list.get_parameters().len(), 1);
    }

    #[test]
    fn trait_get_parameters_boxes_clones() {
        let mut list = DbPlayerMarkersInsertParameterList::new();
        list.add_parameter(DbPlayerMarkersInsertParameter::new(
            "t1".to_string(),
            "p1".to_string(),
            "text".to_string(),
        ));
        let boxed: Vec<Box<dyn IDbUpdateParameter>> = IDbUpdateParameterList::get_parameters(&list);
        assert_eq!(boxed.len(), 1);
        assert_eq!(boxed[0].get_updated_rows(), 0);
    }
}
