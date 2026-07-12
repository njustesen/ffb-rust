/// 1:1 translation of com.fumbbl.ffb.server.request.fumbbl.FumbblRequestLoadTeam.
use ffb_model::model::team::Team;
use ffb_model::xml::XmlHandler;

pub struct FumbblRequestLoadTeam {
    /// Java: `fGameState` — modeled here as its id (`gameState.getId()`), matching the rest
    /// of this crate's `InternalServerCommand*`/`ServerRequest*` convention.
    pub game_id: i64,
    pub coach: String,
    pub team_id: String,
    pub home_team: Option<bool>,
    pub account_properties: Vec<String>,
    request_url: String,
}

impl FumbblRequestLoadTeam {
    pub fn new(
        game_id: i64,
        coach: String,
        team_id: String,
        home_team: Option<bool>,
        account_properties: Vec<String>,
    ) -> Self {
        Self {
            game_id,
            coach,
            team_id,
            home_team,
            account_properties,
            request_url: String::new(),
        }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_account_properties(&self) -> &[String] {
        &self.account_properties
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn set_request_url(&mut self, url: String) {
        self.request_url = url;
    }

    /// Java: `FumbblRequestLoadTeam.process(ServerRequestProcessor)`.
    ///
    /// Fetches FUMBBL_TEAM for [`Self::team_id`] via `team_url_template` and inflates the XML
    /// into a real `Team` (Java: `team = UtilFumbblRequest.loadFumbblTeam(server, getTeamId())`,
    /// where `loadFumbblTeam` XML-parses into a `Team`). Java then treats a missing team, or one
    /// whose XML has no name, as an invalid-team error (`handleInvalidTeam`); that is modeled
    /// here as `Ok(None)` (the caller reports the invalid team), while a valid parse yields
    /// `Ok(Some(team))`.
    ///
    /// On success Java dispatches `InternalServerCommandAddLoadedTeam(gameState, coach, homeTeam,
    /// team, accountProperties)` through `server.getCommunication().handleCommand(...)`. That
    /// redispatch is real in [`QueuedFumbblRequestLoadTeam`] below; this method itself stays a
    /// plain HTTP-fetch-then-parse, matching the other `FumbblRequest*`/`ServerRequest*` types'
    /// split between the "process" method and its `ServerRequest`-trait dispatch adapter.
    pub fn process(
        &mut self,
        client: &dyn super::util_fumbbl_request::HttpClient,
        team_url_template: &str,
    ) -> Result<Option<Team>, String> {
        let url = super::util_fumbbl_request::UtilFumbblRequest::bind(team_url_template, &[self.team_id.as_str()]);
        self.set_request_url(url);
        let team_xml =
            super::util_fumbbl_request::UtilFumbblRequest::load_fumbbl_team(client, team_url_template, &self.team_id)?;
        Ok(team_xml.and_then(|xml| parse_team_xml(&xml)))
    }
}

/// Java: `UtilFumbblRequest.loadFumbblTeam`'s XML→`Team` inflation —
/// `XmlHandler.parse(game, xmlSource, new Team(source))`. Mirrors `team_cache::map_to_team`'s
/// parse-then-downcast pattern (the only real XML→`Team` deserializer in this crate). A team
/// whose XML yields no name is treated as invalid (Java's `!StringTool.isProvided(team.getName())`
/// guard) and reported as `None`.
fn parse_team_xml(xml: &str) -> Option<Team> {
    let empty = Team {
        id: String::new(), name: String::new(), race: String::new(),
        roster_id: String::new(), coach: String::new(),
        rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
        team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
        vampire_lord: false, necromancer: false,
    };
    let parsed = XmlHandler::parse(None, xml, Box::new(empty));
    let team = parsed.into_any().downcast::<Team>().ok().map(|t| *t)?;
    if team.name.is_empty() {
        return None;
    }
    Some(team)
}

/// `ServerRequest` adapter around [`FumbblRequestLoadTeam`] — the Rust shape of Java's
/// `FumbblRequestLoadTeam.process(ServerRequestProcessor)`, following the same
/// `Mutex`-wrapped-request + explicit dispatch-channel pattern as
/// `server_request_load_replay.rs`'s `QueuedServerRequestLoadReplay`. On a successfully
/// parsed team, redispatches the real, already-wired
/// `InternalServerCommandAddLoadedTeam`; on a missing/invalid team, Java calls
/// `handleInvalidTeam` (logs + reports to the coach), which has no wired equivalent here yet,
/// so that branch is a documented no-op rather than fabricated logic.
pub struct QueuedFumbblRequestLoadTeam {
    request: std::sync::Mutex<FumbblRequestLoadTeam>,
    client: std::sync::Arc<dyn super::util_fumbbl_request::HttpClient + Send + Sync>,
    team_url_template: String,
    dispatch_tx: tokio::sync::mpsc::UnboundedSender<crate::model::received_command::ReceivedCommand>,
    session_id: crate::model::received_command::SessionId,
}

impl QueuedFumbblRequestLoadTeam {
    pub fn new(
        request: FumbblRequestLoadTeam,
        client: std::sync::Arc<dyn super::util_fumbbl_request::HttpClient + Send + Sync>,
        team_url_template: impl Into<String>,
        dispatch_tx: tokio::sync::mpsc::UnboundedSender<crate::model::received_command::ReceivedCommand>,
        session_id: crate::model::received_command::SessionId,
    ) -> Self {
        Self {
            request: std::sync::Mutex::new(request),
            client,
            team_url_template: team_url_template.into(),
            dispatch_tx,
            session_id,
        }
    }
}

impl crate::request::server_request::ServerRequest for QueuedFumbblRequestLoadTeam {
    fn process(&self) -> Result<(), String> {
        let (game_id, coach, home_team, account_properties, team) = {
            let mut request = self.request.lock().unwrap();
            let team = request.process(self.client.as_ref(), &self.team_url_template)?;
            (request.game_id, request.coach.clone(), request.home_team, request.account_properties.clone(), team)
        };
        if let Some(team) = team {
            let cmd = crate::net::commands::internal_server_command_add_loaded_team::InternalServerCommandAddLoadedTeam::new(
                game_id, coach, home_team, team, account_properties,
            );
            let _ = self.dispatch_tx.send(crate::model::received_command::ReceivedCommand::new_internal(
                crate::net::commands::any_internal_server_command::AnyInternalServerCommand::AddLoadedTeam(cmd),
                self.session_id,
            ));
        }
        // else: Java's `handleInvalidTeam` — see struct doc comment.
        Ok(())
    }

    fn get_request_url(&self) -> &str {
        ""
    }

    fn set_request_url(&mut self, _url: String) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;
    use crate::request::server_request::ServerRequest;
    use std::sync::Arc;

    #[test]
    fn construct() {
        let r = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        assert_eq!(r.get_coach(), "coach");
        assert_eq!(r.get_team_id(), "team1");
    }

    #[test]
    fn process_returns_parsed_team_when_present() {
        let client = MockHttpClient {
            response: Ok(r#"<team id="team1"><name>The Reavers</name><coach>coach</coach></team>"#.to_string()),
        };
        let mut r = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap().expect("team should parse");
        assert_eq!(r.get_request_url(), "http://fumbbl/team/team1");
        assert_eq!(team.name, "The Reavers");
        assert_eq!(team.id, "team1");
    }

    #[test]
    fn process_returns_none_for_empty_response() {
        let client = MockHttpClient { response: Ok(String::new()) };
        let mut r = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert!(team.is_none());
    }

    #[test]
    fn process_returns_none_for_team_without_name() {
        // Java's `!StringTool.isProvided(team.getName())` invalid-team guard.
        let client = MockHttpClient {
            response: Ok(r#"<team id="team1"><coach>coach</coach></team>"#.to_string()),
        };
        let mut r = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let team = r.process(&client, "http://fumbbl/team/$1").unwrap();
        assert!(team.is_none());
    }

    // ── QueuedFumbblRequestLoadTeam ──────────────────────────────────────────────

    #[test]
    fn queued_dispatches_add_loaded_team_on_success() {
        let client: Arc<dyn super::super::util_fumbbl_request::HttpClient + Send + Sync> = Arc::new(MockHttpClient {
            response: Ok(r#"<team id="team1"><name>The Reavers</name><coach>coach</coach></team>"#.to_string()),
        });
        let (dispatch_tx, mut dispatch_rx) = tokio::sync::mpsc::unbounded_channel();
        let request = FumbblRequestLoadTeam::new(42, "Kalimar".to_string(), "team1".to_string(), Some(true), vec!["ADMIN".to_string()]);
        let queued = QueuedFumbblRequestLoadTeam::new(request, client, "http://fumbbl/team/$1", dispatch_tx, 5);

        queued.process().unwrap();

        let received = dispatch_rx.try_recv().expect("expected a redispatched AddLoadedTeam command");
        assert_eq!(received.session_id, 5);
        match received.command {
            crate::model::received_command::ReceivedNetCommand::Internal(
                crate::net::commands::any_internal_server_command::AnyInternalServerCommand::AddLoadedTeam(cmd),
            ) => {
                assert_eq!(cmd.game_id, 42);
                assert_eq!(cmd.get_coach(), "Kalimar");
                assert_eq!(cmd.get_home_team(), Some(true));
                assert_eq!(cmd.get_team().id, "team1");
                assert_eq!(cmd.get_account_properties(), &["ADMIN".to_string()]);
            }
            _ => panic!("expected an internal AddLoadedTeam command"),
        }
    }

    #[test]
    fn queued_invalid_team_is_a_noop() {
        let client: Arc<dyn super::super::util_fumbbl_request::HttpClient + Send + Sync> =
            Arc::new(MockHttpClient { response: Ok(String::new()) });
        let (dispatch_tx, mut dispatch_rx) = tokio::sync::mpsc::unbounded_channel();
        let request = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let queued = QueuedFumbblRequestLoadTeam::new(request, client, "http://fumbbl/team/$1", dispatch_tx, 1);

        queued.process().unwrap();

        assert!(dispatch_rx.try_recv().is_err());
    }

    #[test]
    fn queued_http_error_propagates() {
        let client: Arc<dyn super::super::util_fumbbl_request::HttpClient + Send + Sync> =
            Arc::new(MockHttpClient { response: Err("network down".to_string()) });
        let (dispatch_tx, mut dispatch_rx) = tokio::sync::mpsc::unbounded_channel();
        let request = FumbblRequestLoadTeam::new(1, "coach".to_string(), "team1".to_string(), Some(true), vec![]);
        let queued = QueuedFumbblRequestLoadTeam::new(request, client, "http://fumbbl/team/$1", dispatch_tx, 1);

        assert!(queued.process().is_err());
        assert!(dispatch_rx.try_recv().is_err());
    }
}
