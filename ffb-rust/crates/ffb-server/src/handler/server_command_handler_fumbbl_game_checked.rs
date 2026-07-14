/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFumbblGameChecked.
use ffb_model::enums::NetCommandId;
use ffb_model::model::roster::Roster;
use ffb_model::xml::{IXmlReadable, XmlHandler};
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
    /// `team_away` are always fully-formed `Team`s, never skeletons — so `inflateIfNeeded` is a
    /// genuine no-op here (there is no skeleton state to inflate from). Phase ZY.2's
    /// `ffb_model::xml::XmlHandler` now lets the roster HTTP response be parsed into a real
    /// `Roster` and applied via `Team::update_roster`, and `GameCache::add_team_to_game`
    /// re-registers both sides exactly as Java does. What remains unported is the tail —
    /// `Game.teamsAreInflated()` (no skeleton/inflated distinction to mark, see above — an
    /// intentional no-op, not a gap), `GameCache.queueDbUpdate` and `UtilServerStartGame.startGame`
    /// (both require the async DB/SessionManager plumbing `util::server_start_game::start_game`
    /// already threads through at its own call sites — wiring an async DB call into this
    /// currently-sync handler is a larger, separately-scoped change, not "just needs a real Team").
    pub fn handle_command(
        &self,
        game_id: i64,
        game_cache: &mut GameCache,
        client: &dyn HttpClient,
        roster_url_template: &str,
    ) -> bool {
        let (home_id, away_id) = {
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
            (game.team_home.id.clone(), game.team_away.id.clone())
        };

        // Java: Roster rosterHome = getRoster(gameState, home.getId());
        //       Roster rosterAway = getRoster(gameState, away.getId());
        //       if (rosterHome == null || rosterAway == null) return false;
        let roster_home = self.get_roster(client, roster_url_template, &home_id, game_id);
        let roster_away = self.get_roster(client, roster_url_template, &away_id, game_id);
        let (Some(roster_home), Some(roster_away)) = (roster_home, roster_away) else {
            return false;
        };

        let game_state = game_cache.get_game_state_by_id_mut(game_id).expect("checked above");
        let game = game_state.get_game_mut().expect("checked above");

        // Java: home.updateRoster(rosterHome, ...); away.updateRoster(rosterAway, ...);
        //       getServer().getGameCache().addTeamToGame(gameState, home, true);
        //       getServer().getGameCache().addTeamToGame(gameState, away, false);
        let mut home = game.team_home.clone();
        home.update_roster(&roster_home);
        GameCache::add_team_to_game(game, home, true);

        let mut away = game.team_away.clone();
        away.update_roster(&roster_away);
        GameCache::add_team_to_game(game, away, false);

        // Java: `gameState.getGame().teamsAreInflated()` — no-op, see doc comment above.
        // Java: `getServer().getGameCache().queueDbUpdate(gameState, true);
        //        UtilServerStartGame.startGame(gameState);` — async DB/SessionManager plumbing,
        // out of scope for this sub-phase (see doc comment above).
        log::debug!(
            "game {}: rosters resolved for home={}/away={} — queueDbUpdate/startGame not wired \
             (async DB/SessionManager plumbing, separate from roster resolution)",
            game_id, home_id, away_id
        );
        true
    }

    /// Java: `private Roster getRoster(GameState gameState, String teamId)`.
    ///
    /// Java rejects both a failed fetch and a fetched-but-unparseable roster (checked via
    /// `!StringTool.isProvided(roster.getName())` — a roster XML without even a `<name>`
    /// tag is treated the same as no roster at all).
    fn get_roster(
        &self,
        client: &dyn HttpClient,
        roster_url_template: &str,
        team_id: &str,
        game_id: i64,
    ) -> Option<Roster> {
        let xml = match UtilFumbblRequest::load_fumbbl_roster_for_team(client, roster_url_template, team_id) {
            Ok(Some(xml)) => xml,
            Ok(None) => {
                // Java: handleInvalidRoster(teamId, gameState, getServer(), null);
                log::error!("game {}: unable to load Roster for Team {}", game_id, team_id);
                return None;
            }
            Err(e) => {
                // Java: handleInvalidRoster(teamId, gameState, getServer(), pFantasyFootballException);
                log::error!("game {}: error loading Roster for Team {}: {}", game_id, team_id, e);
                return None;
            }
        };
        let roster = parse_roster(&xml);
        if roster.name.is_empty() {
            log::error!("game {}: unable to load Roster for Team {}", game_id, team_id);
            return None;
        }
        Some(roster)
    }
}

/// Java: `XmlHandler.parse(gameState.getGame(), xmlSource, new Roster())`.
fn parse_roster(xml: &str) -> Roster {
    let empty = || Roster {
        id: String::new(), name: String::new(), race: String::new(),
        reroll_cost: 0, max_rerolls: 0, positions: vec![], special_rules: vec![],
        necromancer: false, keywords: vec![], raised_position_id: None,
    };
    let parsed = XmlHandler::parse(None, xml, Box::new(empty()));
    match parsed.into_any().downcast::<Roster>() {
        Ok(roster) => *roster,
        Err(_) => empty(),
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

    fn roster_xml(name: &str) -> String {
        format!("<roster id=\"human\"><name>{}</name></roster>", name)
    }

    #[test]
    fn handle_command_missing_gamestate_returns_false() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let client = MockHttpClient { response: Ok(String::new()) };
        assert!(!h.handle_command(999, &mut cache, &client, "http://fumbbl/roster/$1"));
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
        assert!(!h.handle_command(game_id, &mut cache, &client, "http://fumbbl/roster/$1"));
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
        assert!(!h.handle_command(game_id, &mut cache, &client, "http://fumbbl/roster/$1"));
    }

    #[test]
    fn handle_command_unnamed_roster_returns_false() {
        // Java: `!StringTool.isProvided(roster.getName())` rejects a fetched-but-nameless roster.
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);
        let client = MockHttpClient { response: Ok(r#"<roster id="human"/>"#.to_string()) };
        assert!(!h.handle_command(game_id, &mut cache, &client, "http://fumbbl/roster/$1"));
    }

    #[test]
    fn get_roster_returns_parsed_roster_on_success() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let client = MockHttpClient { response: Ok(roster_xml("Human")) };
        let roster = h.get_roster(&client, "http://fumbbl/roster/$1", "t1", 1);
        assert_eq!(roster.map(|r| r.name), Some("Human".to_string()));
    }

    #[test]
    fn get_roster_none_on_empty_team_id() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let client = MockHttpClient { response: Ok(roster_xml("Human")) };
        let roster = h.get_roster(&client, "http://fumbbl/roster/$1", "", 1);
        assert!(roster.is_none());
    }

    #[test]
    fn handle_command_resolves_rosters_and_reregisters_both_teams() {
        let h = ServerCommandHandlerFumbblGameChecked::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);

        let responses = std::cell::RefCell::new(vec![roster_xml("Away Roster"), roster_xml("Home Roster")]);
        struct SequencedClient(std::cell::RefCell<Vec<String>>);
        impl HttpClient for SequencedClient {
            fn fetch_page(&self, _url: &str) -> Result<String, String> {
                Ok(self.0.borrow_mut().pop().unwrap_or_default())
            }
        }
        let client = SequencedClient(responses);

        assert!(h.handle_command(game_id, &mut cache, &client, "http://fumbbl/roster/$1"));

        let gs = cache.get_game_state_by_id(game_id).unwrap();
        let game = gs.get_game().unwrap();
        assert_eq!(game.team_home.roster_id, "human");
        assert_eq!(game.team_home.race, "Home Roster");
        assert_eq!(game.team_away.race, "Away Roster");
    }
}
