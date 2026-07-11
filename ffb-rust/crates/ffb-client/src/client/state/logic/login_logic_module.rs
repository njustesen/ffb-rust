//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.LoginLogicModule` (218 lines).
//!
//! Java's `LoginLogicModule` drives the coach-facing login/join handshake: version negotiation,
//! password-challenge/response, and the final `sendJoin` call (either joining a specific game or
//! browsing the game list). Methods needing `FantasyFootballClient` access are translated as
//! inherent methods taking `client` explicitly (matching the established `MoveLogicModule`
//! convention), since the `LogicModule` trait's own interaction-method defaults have a narrower
//! signature (see `logic_module.rs`'s module doc).
//!
//! Documented gaps:
//! - `createResponse(String pChallenge)` calls `com.fumbbl.ffb.PasswordChallenge.createResponse`
//!   — an MD5-based HMAC-style challenge-response helper (`ffb-common/.../PasswordChallenge.java`,
//!   distinct from the already-translated `ffb_model::model::password_challenge::PasswordChallenge`
//!   data holder). No MD5 primitive is available in this crate's dependencies
//!   (`ffb-client/Cargo.toml` has no crypto crate), so this cannot be computed faithfully;
//!   conservatively returns `None`, mirroring Java's own `catch (IOException |
//!   NoSuchAlgorithmException ioe) { response = null; }` fallback.
//! - `handleVersionCommand`'s per-property loop (`client.setProperty(property,
//!   pNetCommand.getClientPropertyValue(property))`) — `FantasyFootballClient::set_property` is
//!   `abstract` in Java with no in-scope concrete body (see `fantasy_football_client.rs`'s module
//!   doc, "NOT translated" list), so this loop is a documented no-op here.
//! - `Version`'s regex-based parsing (`Pattern.compile("([0-9]+)\\.([0-9]+)\\.([0-9]+)")`) has no
//!   `regex` crate dependency available in this workspace; translated as an equivalent
//!   3-part-dot-split parse that only populates `major`/`minor`/`release` on a full match,
//!   exactly matching `Matcher.matches()`'s all-or-nothing semantics (partial/non-numeric input
//!   leaves the fields at their `0` default, same as Java's unpopulated fields when the pattern
//!   doesn't match).
//! - `sendJoin(...)`'s `null` `gameName`/`teamId`/`teamName` arguments (list-games branch) — the
//!   Rust `ClientCommunication::send_join` takes non-optional `impl Into<String>` parameters (no
//!   "null" variant), so empty strings are substituted, matching the same documented-gap pattern
//!   as `LogicModule::deselect_acting_player`.
//! - `getClient().getMode()` (implicit `this.client` read in Java's own inherited
//!   `FantasyFootballClient client` field access for `sendJoin`) is taken as `client.mode()`
//!   here, defaulting to `ClientMode::PLAYER` if unset (mirrors the "player" login flow this
//!   module exists for).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::client_mode::ClientMode;
use ffb_model::model::game::Game;
use ffb_model::model::game_list_entry::GameListEntry;
use ffb_model::model::player::Player;
use ffb_model::model::team_list_entry::TeamListEntry;
use ffb_model::util::string_tool;
use ffb_protocol::commands::server_command_password_challenge::ServerCommandPasswordChallenge;
use ffb_protocol::commands::server_command_version::ServerCommandVersion;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `LoginLogicModule.VersionCheck`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCheck {
    Success,
    ClientFail,
    ServerFail,
}

/// java: `LoginLogicModule.LoginData`.
#[derive(Debug, Clone, Default)]
pub struct LoginData {
    pub game_name: Option<String>,
    pub encoded_password: Option<Vec<u8>>,
    pub password_length: i32,
    pub list_games: bool,
}

impl LoginData {
    pub fn new(
        game_name: Option<String>,
        encoded_password: Option<Vec<u8>>,
        password_length: i32,
        list_games: bool,
    ) -> Self {
        Self { game_name, encoded_password, password_length, list_games }
    }
}

/// java: `private static class Version` — see module doc regarding regex parsing.
struct Version {
    major: i32,
    minor: i32,
    release: i32,
}

impl Version {
    fn parse(version: &str) -> Self {
        let mut result = Version { major: 0, minor: 0, release: 0 };
        let parts: Vec<&str> = version.split('.').collect();
        let is_full_numeric_match = parts.len() == 3
            && parts.iter().all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
        if is_full_numeric_match {
            result.major = parts[0].parse().unwrap_or(0);
            result.minor = parts[1].parse().unwrap_or(0);
            result.release = parts[2].parse().unwrap_or(0);
        }
        result
    }
}

/// 1:1 translation of the `LoginLogicModule` class.
#[derive(Debug, Default)]
pub struct LoginLogicModule {
    game_name: Option<String>,
    encoded_password: Option<Vec<u8>>,
    password_length: i32,
    list_games: bool,
    team_home_id: Option<String>,
    team_home_name: Option<String>,
    team_away_name: Option<String>,
    game_id: i64,
}

impl LoginLogicModule {
    /// java: `public LoginLogicModule(FantasyFootballClient client)`.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_team_home_name(&self) -> Option<&str> {
        self.team_home_name.as_deref()
    }

    pub fn get_team_away_name(&self) -> Option<&str> {
        self.team_away_name.as_deref()
    }

    pub fn get_game_name(&self) -> Option<&str> {
        self.game_name.as_deref()
    }

    pub fn get_encoded_password(&self) -> Option<&[u8]> {
        self.encoded_password.as_deref()
    }

    pub fn get_password_length(&self) -> i32 {
        self.password_length
    }

    pub fn set_password_length(&mut self, password_length: i32) {
        self.password_length = password_length;
    }

    /// java: `public void initCommunication()`.
    pub fn init_communication(&mut self, client: &mut FantasyFootballClient) {
        if string_tool::is_provided(client.parameters().team_id()) {
            self.team_home_id = client.parameters().team_id().map(str::to_string);
            self.team_home_name = client.parameters().team_name().map(str::to_string);
            self.team_away_name = None;
        } else {
            self.team_home_id = None;
            self.team_home_name = client.parameters().team_home().map(str::to_string);
            self.team_away_name = client.parameters().team_away().map(str::to_string);
        }
        client.communication_mut().send_request_version();
    }

    /// java: `public void sendChallenge(LoginData loginData)`.
    pub fn send_challenge_from_login_data(&mut self, client: &mut FantasyFootballClient, login_data: LoginData) {
        self.encoded_password = login_data.encoded_password;
        self.game_name = login_data.game_name;
        self.list_games = login_data.list_games;
        self.password_length = login_data.password_length;
        self.send_challenge(client);
    }

    /// java: `public void sendChallenge(TeamListEntry teamListEntry)`.
    pub fn send_challenge_from_team_list_entry(
        &mut self,
        client: &mut FantasyFootballClient,
        team_list_entry: &TeamListEntry,
    ) {
        self.team_home_id = Some(team_list_entry.get_team_id().to_string());
        self.team_home_name = Some(team_list_entry.get_team_name().to_string());
        self.send_challenge(client);
    }

    /// java: `public void sendChallenge(GameListEntry gameListEntry)`.
    pub fn send_challenge_from_game_list_entry(
        &mut self,
        client: &mut FantasyFootballClient,
        game_list_entry: &GameListEntry,
    ) {
        self.game_id = game_list_entry.get_game_id() as i64;
        self.list_games = false;
        self.send_challenge(client);
    }

    /// java: `private void sendChallenge()`.
    fn send_challenge(&mut self, client: &mut FantasyFootballClient) {
        let authentication = client.parameters().authentication().map(str::to_string);
        if string_tool::is_provided(authentication.as_deref()) {
            let authentication = authentication.unwrap();
            self.send_join(client, authentication);
        } else {
            let coach = client.parameters().coach().unwrap_or_default().to_string();
            client.communication_mut().send_password_challenge(coach);
        }
    }

    /// java: `public void sendJoin(String pResponse)` — see module doc regarding the
    /// `null` `gameName`/`teamId`/`teamName` arguments and implicit `getClient().getMode()`.
    pub fn send_join(&self, client: &mut FantasyFootballClient, response: impl Into<String>) {
        let response = response.into();
        let mode = client.mode().unwrap_or(ClientMode::PLAYER);
        let coach = client.parameters().coach().unwrap_or_default().to_string();
        if self.list_games {
            client
                .communication_mut()
                .send_join(mode, coach, response, 0, String::new(), String::new(), String::new());
        } else {
            let game_id = if self.game_id > 0 { self.game_id } else { client.parameters().game_id() };
            let game_name = self.game_name.clone().unwrap_or_default();
            let team_home_id = self.team_home_id.clone().unwrap_or_default();
            let team_home_name = self.team_home_name.clone().unwrap_or_default();
            client
                .communication_mut()
                .send_join(mode, coach, response, game_id, game_name, team_home_id, team_home_name);
        }
    }

    /// java: `public void handlePasswordChallenge(ServerCommandPasswordChallenge pNetCommand)`.
    pub fn handle_password_challenge(
        &self,
        client: &mut FantasyFootballClient,
        net_command: &ServerCommandPasswordChallenge,
    ) {
        let response = self.create_response(net_command.get_challenge());
        if let Some(response) = response {
            self.send_join(client, response);
        } else {
            // java: gap — see module doc comment (`createResponse` has no MD5 primitive
            // available); Java would still call `sendJoin(null)` here, but the Rust
            // `send_join` requires a non-optional response string, so this is skipped.
        }
    }

    /// java: `public boolean checkVersionConflict(String pVersionExpected, String pVersionIs)`.
    pub fn check_version_conflict(&self, version_expected: &str, version_is: &str) -> bool {
        let expected_version = Version::parse(version_expected);
        let actual_version = Version::parse(version_is);
        (actual_version.major < expected_version.major)
            || (actual_version.minor < expected_version.minor)
            || (actual_version.release < expected_version.release)
    }

    /// java: `private String createResponse(String pChallenge)` — see module doc gap.
    fn create_response(&self, _challenge: &str) -> Option<String> {
        // java: gap — see module doc comment (no MD5 primitive available in this crate).
        None
    }

    /// java: `public boolean idAndNameProvided()`.
    pub fn id_and_name_provided(&self, client: &FantasyFootballClient) -> bool {
        client.parameters().game_id() == 0 && string_tool::is_provided(self.get_game_name())
    }

    /// java: `public VersionCheck handleVersionCommand(ServerCommandVersion pNetCommand)`.
    pub fn handle_version_command(
        &self,
        client: &mut FantasyFootballClient,
        net_command: &ServerCommandVersion,
    ) -> VersionCheck {
        let version_check = self.check_version(net_command.get_server_version(), net_command.get_client_version());
        if net_command.is_test_server() || version_check == VersionCheck::Success {
            // java: gap — see module doc comment (`client.setProperty` is abstract with no
            // in-scope concrete body); the per-property loop is a documented no-op.
            let _ = client;
        }
        version_check
    }

    /// java: `private VersionCheck checkVersion(String pServerVersion, String pClientVersion)`.
    fn check_version(&self, server_version: &str, client_version: &str) -> VersionCheck {
        if self.check_version_conflict(client_version, ffb_model::types::constants::VERSION) {
            return VersionCheck::ClientFail;
        }
        if self.check_version_conflict(ffb_model::types::constants::VERSION, server_version) {
            return VersionCheck::ServerFail;
        }
        VersionCheck::Success
    }
}

impl LogicModule for LoginLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Login
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always
    /// throws `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in login context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn make_client(args: &[&str]) -> FantasyFootballClient {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&args).unwrap();
        FantasyFootballClient::new(params)
    }

    #[test]
    fn get_id_is_login() {
        assert_eq!(LoginLogicModule::new().get_id(), ClientStateId::Login);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(LoginLogicModule::new().available_actions().is_empty());
    }

    #[test]
    fn check_version_conflict_true_when_actual_major_lower() {
        let module = LoginLogicModule::new();
        assert!(module.check_version_conflict("2.0.0", "1.9.9"));
    }

    #[test]
    fn check_version_conflict_false_when_equal() {
        let module = LoginLogicModule::new();
        assert!(!module.check_version_conflict("1.2.3", "1.2.3"));
    }

    #[test]
    fn check_version_conflict_false_on_unparseable_version() {
        let module = LoginLogicModule::new();
        // java: unmatched regex leaves both major/minor/release at 0, so no conflict.
        assert!(!module.check_version_conflict("not-a-version", "also-not"));
    }

    #[test]
    fn create_response_is_always_none() {
        let module = LoginLogicModule::new();
        assert!(module.create_response("abc123").is_none());
    }

    #[test]
    fn init_communication_uses_team_id_when_provided() {
        let mut module = LoginLogicModule::new();
        let mut client = make_client(&["-player", "-coach", "bob", "-teamId", "42", "-teamName", "Orcs"]);
        module.init_communication(&mut client);
        assert_eq!(module.team_home_id, Some("42".to_string()));
        assert_eq!(module.team_home_name, Some("Orcs".to_string()));
        assert!(module.team_away_name.is_none());
    }

    #[test]
    fn init_communication_uses_team_home_and_away_without_team_id() {
        let mut module = LoginLogicModule::new();
        let mut client = make_client(&[
            "-player", "-coach", "bob", "-teamHome", "Orcs", "-teamAway", "Elves",
        ]);
        module.init_communication(&mut client);
        assert!(module.team_home_id.is_none());
        assert_eq!(module.team_home_name, Some("Orcs".to_string()));
        assert_eq!(module.team_away_name, Some("Elves".to_string()));
    }

    #[test]
    fn id_and_name_provided_requires_zero_game_id_and_game_name() {
        let mut module = LoginLogicModule::new();
        let client = make_client(&["-player", "-coach", "bob"]);
        assert!(!module.id_and_name_provided(&client));
        module.game_name = Some("LocalGame".to_string());
        assert!(module.id_and_name_provided(&client));
    }

    #[test]
    fn handle_version_command_success_when_versions_match() {
        let module = LoginLogicModule::new();
        let mut client = make_client(&["-player", "-coach", "bob"]);
        let cmd = ServerCommandVersion::new(
            ffb_model::types::constants::VERSION,
            ffb_model::types::constants::VERSION,
            Default::default(),
            false,
        );
        assert_eq!(module.handle_version_command(&mut client, &cmd), VersionCheck::Success);
    }

    #[test]
    fn handle_version_command_client_fail_when_client_too_old() {
        // java: `checkVersion` fails as CLIENT_FAIL when this crate's own build version is
        // lower than the minimum client version the server-reported command demands.
        let module = LoginLogicModule::new();
        let mut client = make_client(&["-player", "-coach", "bob"]);
        let cmd = ServerCommandVersion::new(
            ffb_model::types::constants::VERSION,
            "99.0.0",
            Default::default(),
            false,
        );
        assert_eq!(module.handle_version_command(&mut client, &cmd), VersionCheck::ClientFail);
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in login context")]
    fn action_context_panics() {
        let module = LoginLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
