//! 1:1 translation of `com.fumbbl.ffb.client.FantasyFootballClient` (`abstract`).
//!
//! DEVIATION FROM THIS PROJECT'S PRIOR CLASSIFICATION: `FantasyFootballClient` was previously
//! treated as a permanently-skipped GUI shell (see the doc comments on `client/net`,
//! `client/handler`). A call-site census across all 85 `client/state/` files found ~1,000 hits
//! on `getGame()`/`getCommunication()`/`getClientData()`/`getParameters()` — far too pervasive
//! for the explicit-parameter trick used in the handler layer. This type is therefore promoted
//! to hold the logic-relevant subset of Java's fields as real, owned data. Genuinely Swing-only
//! surface (session/socket plumbing, `abstract` methods whose only bodies live in the AWT client
//! or in `ffb-ai`, neither of which this project translates) stays undefined, not stubbed with
//! invented logic, per `CLAUDE.md`'s "no invented logic" rule. This does not require reworking
//! `client/net`/`client/handler`'s existing lower-coupling design, which remains valid for that
//! layer.
//!
//! NOT translated (all `abstract` in Java with no in-scope concrete body — their only bodies
//! live in the AWT client, permanently GUI-skip, or in `ffb-ai`, a module this project does not
//! translate at all per `CLAUDE.md`'s crate table): `getUserInterface()`, `getActionKeyBindings()`,
//! `dialogClosed()`, `postConnect()`/`preConnect()`, `getProperty()`/`setProperty()` (both
//! overloads), `initUI()`, `getServerPort()`/`getServerHost()`, `loadProperties()`,
//! `getLocallyStoredPropertyKeys()`/`setLocallyStoredPropertyKeys()`, `loadLocallyStoredProperties()`,
//! `exit()`, `clearPrefs()`/`setPref()`, `getActiveOverlay()`/`setActiveOverlay()`,
//! `replayInitialized()`, `getOverlays()`, `initRulesDependentMembers()`. Their call sites in
//! `client/state/` (once translated) are handled per-file with a
//! `// java: abstract, no in-scope body, skipped` comment, not invented logic.
//!
//! Also not translated: the constructor's `FactoryManager`/`factories` wiring and
//! `getFactorySource()`/`forContext()`/`getFactory()` (`IFactorySource` impl) — the ported
//! `ffb_model::model::Game::new(home, away, rules)` has a different, simpler signature than
//! Java's `Game(IFactorySource, FactoryManager)`, so there is no 1:1 constructor to translate;
//! `game` starts `None` here and is populated by whoever constructs a `Game` for a running
//! session (a later, wider integration concern), not by this constructor. Likewise
//! `initConnection()`/`closeConnection()` (real `javax.websocket` session lifecycle) are not
//! ported — `crate::connection::ServerConnection` already covers real socket I/O on a different
//! async stack, matching the precedent set by `client/net/command_endpoint.rs`.

use ffb_model::model::{ClientMode, Game};

use crate::client::client_data::ClientData;
use crate::client::client_parameters::ClientParameters;
use crate::client::handler::client_command_handler_factory::ClientCommandHandlerFactory;
use crate::client::net::client_communication::ClientCommunication;
use crate::client::net::command_endpoint::CommandEndpoint;

/// Java: `com.fumbbl.ffb.client.FantasyFootballClient`.
pub struct FantasyFootballClient {
    /// Java: `fClientData`.
    client_data: ClientData,
    /// Java: `fGame`. `None` before a game is constructed for this client (see module doc).
    game: Option<Game>,
    /// Java: `fMode`.
    mode: Option<ClientMode>,
    /// Java: `parameters`.
    parameters: ClientParameters,
    /// Java: `fCommandHandlerFactory`.
    command_handler_factory: ClientCommandHandlerFactory,
    /// Java: `fCommunication`.
    communication: ClientCommunication,
    /// Java: `fCommandEndpoint`.
    command_endpoint: CommandEndpoint,
    /// Java: `Boolean.parseBoolean(getProperty(CommonProperty.CLIENT_DEBUG_STATE))`, read inside
    /// `updateClientState()`. `getProperty()` is `abstract` with no in-scope body (see module
    /// doc), so this is represented as a plain field instead, per this project's established
    /// "Swing/abstract-store type becomes plain data" convention.
    debug_client_state: bool,
}

impl FantasyFootballClient {
    /// Java: `public FantasyFootballClient(ClientParameters parameters) throws IOException`.
    /// `loadProperties()`/`loadLocallyStoredProperties()` (abstract, no in-scope body) and the
    /// `FactoryManager`/`Game` construction (see module doc) are not ported; `fCommunicationThread`
    /// (Java starts the communication loop on its own thread here) is this project's caller's
    /// responsibility, matching `ClientCommunication`'s own doc note that it is driven externally.
    pub fn new(parameters: ClientParameters) -> Self {
        let mode = parameters.mode();
        Self {
            client_data: ClientData::new(),
            game: None,
            mode,
            parameters,
            command_handler_factory: ClientCommandHandlerFactory::new(),
            communication: ClientCommunication::new(),
            command_endpoint: CommandEndpoint::default(),
            debug_client_state: false,
        }
    }

    /// Java: `public long gameId()`.
    pub fn game_id(&self) -> i64 {
        self.game.as_ref().map(|game| game.id as i64).unwrap_or(0)
    }

    /// Java: `public ClientCommunication getCommunication()`.
    pub fn communication(&self) -> &ClientCommunication {
        &self.communication
    }

    pub fn communication_mut(&mut self) -> &mut ClientCommunication {
        &mut self.communication
    }

    /// Java: `public ClientParameters getParameters()`.
    pub fn parameters(&self) -> &ClientParameters {
        &self.parameters
    }

    /// Java: `public ClientData getClientData()`.
    pub fn client_data(&self) -> &ClientData {
        &self.client_data
    }

    pub fn client_data_mut(&mut self) -> &mut ClientData {
        &mut self.client_data
    }

    /// Java: `public CommandEndpoint getCommandEndpoint()`.
    pub fn command_endpoint(&self) -> &CommandEndpoint {
        &self.command_endpoint
    }

    /// Java: `public ClientCommandHandlerFactory getCommandHandlerFactory()`.
    pub fn command_handler_factory(&self) -> &ClientCommandHandlerFactory {
        &self.command_handler_factory
    }

    /// Java: `public Game getGame()`.
    pub fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    /// Mutable counterpart of `game()`, matching the `communication()`/`communication_mut()`
    /// convention already established on this struct — needed by `LogicModule` default
    /// methods that mutate field-model state (e.g. `setRangeRuler(null)`).
    pub fn game_mut(&mut self) -> Option<&mut Game> {
        self.game.as_mut()
    }

    /// Java: `public void setGame(Game pGame) { fGame = pGame; getClientData().clear(); }`.
    pub fn set_game(&mut self, game: Game) {
        self.game = Some(game);
        self.client_data.clear();
    }

    /// Java: `public ClientMode getMode()`.
    pub fn mode(&self) -> Option<ClientMode> {
        self.mode
    }

    /// Java: `public void setMode(ClientMode pMode)`.
    pub fn set_mode(&mut self, mode: ClientMode) {
        self.mode = Some(mode);
    }

    pub fn is_debug_client_state(&self) -> bool {
        self.debug_client_state
    }

    pub fn set_debug_client_state(&mut self, debug: bool) {
        self.debug_client_state = debug;
    }

    /// Java: `public void logError(String message) { logError(gameId(), message); }`. Routes
    /// through the `log` crate; the two-argument `logError(long, String)` overload (game-id
    /// tagged logging) has no in-scope concrete body to translate (abstract in Java), so the
    /// game id is folded into the log target here instead.
    pub fn log_error(&self, message: &str) {
        log::error!(target: "ffb_client", "[game {}] {}", self.game_id(), message);
    }

    /// Java: `public void logDebug(String message) { logDebug(gameId(), message); }`.
    pub fn log_debug(&self, message: &str) {
        log::debug!(target: "ffb_client", "[game {}] {}", self.game_id(), message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parameters_with_mode(mode: ClientMode) -> ClientParameters {
        let args: Vec<String> = match mode {
            ClientMode::PLAYER => vec!["-player".into(), "-coach".into(), "bob".into()],
            ClientMode::SPECTATOR => vec!["-spectator".into(), "-coach".into(), "bob".into()],
            ClientMode::REPLAY => vec!["-replay".into(), "-gameId".into(), "1".into()],
        };
        ClientParameters::create_valid_params(&args).unwrap()
    }

    #[test]
    fn new_has_no_game_and_zero_game_id() {
        let client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        assert!(client.game().is_none());
        assert_eq!(client.game_id(), 0);
    }

    #[test]
    fn new_takes_mode_from_parameters() {
        let client = FantasyFootballClient::new(parameters_with_mode(ClientMode::REPLAY));
        assert_eq!(client.mode(), Some(ClientMode::REPLAY));
    }

    #[test]
    fn set_mode_overrides_mode() {
        let mut client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        client.set_mode(ClientMode::PLAYER);
        assert_eq!(client.mode(), Some(ClientMode::PLAYER));
    }

    fn make_team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    #[test]
    fn set_game_clears_client_data() {
        let mut client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        client.client_data_mut().set_end_turn_button_hidden(true);
        let game = Game::new(make_team("home"), make_team("away"), ffb_model::enums::Rules::Bb2025);
        client.set_game(game);
        assert!(!client.client_data().is_end_turn_button_hidden());
    }

    #[test]
    fn game_id_reflects_set_game() {
        let mut client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        assert_eq!(client.game_id(), 0);
        let mut game = Game::new(make_team("home"), make_team("away"), ffb_model::enums::Rules::Bb2025);
        game.id = 7;
        client.set_game(game);
        assert_eq!(client.game_id(), 7);
    }

    #[test]
    fn debug_client_state_round_trips() {
        let mut client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        assert!(!client.is_debug_client_state());
        client.set_debug_client_state(true);
        assert!(client.is_debug_client_state());
    }

    #[test]
    fn communication_is_accessible_and_mutable() {
        let mut client = FantasyFootballClient::new(parameters_with_mode(ClientMode::SPECTATOR));
        assert!(!client.communication().is_stopped());
        client.communication_mut().stop();
        assert!(client.communication().is_stopped());
    }
}
