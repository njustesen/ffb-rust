/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFumbblGameChecked.
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::request::fumbbl::util_fumbbl_request::{HttpClient, UtilFumbblRequest};

pub struct ServerCommandHandlerFumbblGameChecked;

impl ServerCommandHandlerFumbblGameChecked {
    pub fn new() -> Self {
        Self
    }

    /// Java: getId() — returns NetCommandId for FUMBBL_GAME_CHECKED.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerFumbblGameChecked
    }

    /// Java: handleCommand(ReceivedCommand) — handles FUMBBL game-checked notification.
    ///
    /// Java: inflates each `TeamSkeleton` into a full `Team` via `XmlHandler.parse`, fetches each
    /// team's `Roster` over HTTP (`UtilFumbblRequest.loadFumbblRosterForTeam`), applies the
    /// roster (`Team.updateRoster`), re-registers both teams in the `GameCache`
    /// (`addTeamToGame`), marks the teams inflated, queues a DB update, and finally starts the
    /// game (`UtilServerStartGame.startGame`).
    ///
    /// The Rust model has no `TeamSkeleton`/`Team` duality — a `GameState`'s `team_home`/
    /// `team_away` are always fully-formed `Team`s, never skeletons — so there is no XML team
    /// parser or `Team::update_roster` to call, and `GameCache::add_team_to_game` /
    /// `UtilServerStartGame::start_game` are not wired (the latter is explicitly documented as
    /// skipped in `ffb_engine::util::util_server_start_game`, being DB/WebSocket/SessionManager/
    /// SequenceGenerator dependent). The one piece that *can* be ported for real — the roster
    /// HTTP fetch and its error handling — is implemented below via `get_roster`; everything
    /// past it is narrowly gated behind that missing infra.
    pub fn handle_command(
        &self,
        game_id: i64,
        game_cache: &GameCache,
        client: &dyn HttpClient,
        roster_url_template: &str,
    ) -> bool {
        let game_state = match game_cache.get_game_state_by_id(game_id) {
            Some(gs) => gs,
            None => {
                log::error!("game {}: gamestate not found for FumbblGameChecked", game_id);
                return false;
            }
        };

        let game = match game_state.get_game() {
            Some(g) => g,
            None => {
                log::error!("game {}: game not started for FumbblGameChecked", game_id);
                return false;
            }
        };

        // Java: Roster rosterHome = getRoster(gameState, home.getId());
        //       Roster rosterAway = getRoster(gameState, away.getId());
        //       if (rosterHome == null || rosterAway == null) return false;
        let roster_home = self.get_roster(client, roster_url_template, &game.team_home.id, game_id);
        let roster_away = self.get_roster(client, roster_url_template, &game.team_away.id, game_id);
        if roster_home.is_none() || roster_away.is_none() {
            return false;
        }

        // Java: home.updateRoster(rosterHome, ...); away.updateRoster(rosterAway, ...);
        //       getServer().getGameCache().addTeamToGame(gameState, home, true);
        //       getServer().getGameCache().addTeamToGame(gameState, away, false);
        //       gameState.getGame().teamsAreInflated();
        //       getServer().getGameCache().queueDbUpdate(gameState, true);
        //       UtilServerStartGame.startGame(gameState);
        todo!(
            "Phase ZV: needs Team::update_roster + GameCache::add_team_to_game + \
             UtilServerStartGame::start_game, not yet wired"
        )
    }

    /// Java: `private Roster getRoster(GameState gameState, String teamId)`.
    fn get_roster(
        &self,
        client: &dyn HttpClient,
        roster_url_template: &str,
        team_id: &str,
        game_id: i64,
    ) -> Option<String> {
        match UtilFumbblRequest::load_fumbbl_roster_for_team(client, roster_url_template, team_id) {
            Ok(Some(xml)) => Some(xml),
            Ok(None) => {
                // Java: handleInvalidRoster(teamId, gameState, getServer(), null);
                log::error!("game {}: unable to load Roster for Team {}", game_id, team_id);
                None
            }
            Err(e) => {
                // Java: handleInvalidRoster(teamId, gameState, getServer(), pFantasyFootballException);
                log::error!("game {}: error loading Roster for Team {}: {}", game_id, team_id, e);
                None
            }
        }
    }
}

impl Default for ServerCommandHandlerFumbblGameChecked {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn team(id: &str) -> Team {
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerFumbblGameChecked::new();
    }

    #[test]
    fn get_id_is_fumbbl_game_checked() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        assert_eq!(h.get_id(), NetCommandId::InternalServerFumbblGameChecked);
    }

    #[test]
    fn handle_command_missing_gamestate_returns_false() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let cache = GameCache::new();
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(!h.handle_command(999, &cache, &client, "http://fumbbl/roster/$1"));
    }

    #[test]
    fn handle_command_missing_roster_returns_false() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(!h.handle_command(game_id, &cache, &client, "http://fumbbl/roster/$1"));
    }

    #[test]
    fn handle_command_http_error_returns_false() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);
        let client = MockHttpClient { response: Err("connection refused".to_string()) };
        assert!(!h.handle_command(game_id, &cache, &client, "http://fumbbl/roster/$1"));
    }

    #[test]
    fn get_roster_returns_xml_on_success() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let client = MockHttpClient { response: Ok("<roster/>".to_string()) };
        let roster = h.get_roster(&client, "http://fumbbl/roster/$1", "t1", 1);
        assert_eq!(roster, Some("<roster/>".to_string()));
    }

    #[test]
    fn get_roster_none_on_empty_team_id() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let client = MockHttpClient { response: Ok("<roster/>".to_string()) };
        let roster = h.get_roster(&client, "http://fumbbl/roster/$1", "", 1);
        assert_eq!(roster, None);
    }

    #[test]
    fn roster_fetch_used_by_both_teams_before_todo() {
        // Sanity check that a game with an unfetchable roster for either side short-circuits
        // before reaching the (currently unwired) team-registration/start-game step.
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        let game = Game::new(team("home"), team("away"), Rules::Bb2025);
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(game.team_home.clone(), game.team_away.clone(), Rules::Bb2025, 0);
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(!h.handle_command(game_id, &cache, &client, "http://fumbbl/roster/$1"));
    }
}
