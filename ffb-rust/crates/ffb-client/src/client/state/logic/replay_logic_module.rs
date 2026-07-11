//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.ReplayLogicModule`.
//!
//! Handles the `REPLAY` client state: loading a stored/live replay command stream from the
//! server, tracking control-of-replay handoff between spectating coaches, and driving the
//! `ReplayCallbacks` lifecycle hooks a UI layer implements to react to load progress.
//!
//! DOCUMENTED GAPS (this module has more of these than the batch's other four, because Java's
//! `ReplayLogicModule` leans heavily on infrastructure this project has not yet translated):
//! - `client.getReplayer()` (`ClientReplayer`) and `getReplayControl()` (`ReplayControl`) are
//!   both still trivial untranslated PascalCase stubs (`client/ClientReplayer.rs`,
//!   `client/ReplayControl.rs`) — every call through them (`replayer.start()`,
//!   `setControl(...)`, `positionOnFirstCommand()`/`positionOnLastCommand()`, `init(...)`,
//!   `addMarkingConfigs(...)`, `isOnline()`, `setOnline(...)`, `getReplayControl().setActive(...)`
//!   /`.refresh()`) has no real target and is left as a documented no-op at its call site.
//! - `client.getProperty(CommonProperty...)` / `IClientPropertyValue.AUTO_MARKING` — `getProperty`
//!   is `abstract` with no in-scope body (see `fantasy_football_client.rs` module doc), and no
//!   `AUTO_MARKING` value set was ever ported alongside `IClientPropertyValue`, so the
//!   auto-marking seed logic in `setUp()` is left undone (conservatively: no seed entry added).
//! - `client.getOverlays()` (`FantasyFootballClient.getOverlays()`) is not translated (see that
//!   file's module doc) — the `ControlAware`/`OnlineAware` overlay-notification loops in
//!   `evaluateControl`/`replayMode` have no overlay collection to iterate and are skipped.
//! - `client.replayInitialized()` is `abstract` with no in-scope body — skipped at its call site.
//! - The translated `ffb_protocol::server_commands::ServerCommand` enum (see that file's own
//!   module doc: "a not-1:1 simplification") only has variants for a subset of Java's
//!   `ServerCommand*` types (`ServerGameState`, `ServerStatus`, `ServerJoin`, `ServerLeave`, plus
//!   a handful more) and none of them carry the payload fields `ReplayLogicModule.handleCommand`
//!   actually reads (`ServerCommandReplay.getReplayCommands()`/`getTotalNrOfCommands()`, etc. for
//!   `SERVER_REPLAY`; `ServerCommandAutomaticPlayerMarkings` for `SERVER_AUTOMATIC_PLAYER_MARKINGS`;
//!   `ServerCommandReplayStatus`/`ServerCommandReplayControl` for their cases; and even the
//!   modeled `ServerJoin`/`ServerLeave`/`ServerStatus` variants carry different fields than
//!   Java's `ServerCommandJoin`/`ServerCommandLeave`/`ServerCommandStatus` — no `spectators`/
//!   `replayName` on `ServerJoin`, no connection-level `ServerStatus` enum payload on
//!   `ServerStatus`). `handle_command` below therefore only implements the one case whose
//!   behavior needs no such payload data (`SERVER_GAME_STATE`); every other `NetCommandId` match
//!   arm is a documented no-op gap.

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::ClientMode;
use ffb_model::util::string_tool;
use ffb_protocol::server_commands::ServerCommand;

use crate::client::action_key::ActionKey;
use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::i_progress_listener::IProgressListener;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `Constant.REPLAY_NAME_MAX_LENGTH` — `Constant` (`ffb-common`) is not translated in this
/// project; the literal value (20) is reproduced here since it's the only member of that class
/// this file needs.
const REPLAY_NAME_MAX_LENGTH: usize = 20;

/// 1:1 translation of `ReplayLogicModule.ReplayCallbacks` (Java nested interface).
///
/// Implement this to react to life cycle events during replay loading. You must pass an
/// instance to [`ReplayLogicModule::set_callbacks`], otherwise loading replays will fail (mirrors
/// Java's own doc comment on the interface).
pub trait ReplayCallbacks {
    /// Called when the list of replay commands is truncated.
    fn reset(&mut self);
    /// Called when we get the total number of replay commands that will be loaded from the
    /// server.
    fn command_count(&mut self, total_commands: i32);
    /// Called during command load sequence with the current number of commands loaded so far.
    fn loaded_commands(&mut self, current_size: i32);
    /// Marks when all commands have been loaded from the server.
    fn load_done(&mut self);
    /// Called before the replayer will be initialized.
    fn start_replayer_init(&mut self);
    /// Called after replayer is initialized.
    fn replayer_initialized(&mut self);
    /// The returned listener is passed to the replayer to be used during initialization.
    fn progress_listener(&mut self) -> Box<dyn IProgressListener>;
    /// Called if the replay was not found on server side, connection will be closed.
    fn replay_unavailable(&mut self, status: ffb_model::enums::ServerStatus);
    /// Called when the control state of the client changes.
    fn control_changed(&mut self, controlling_coach: &str);
    /// Called when a replay state command was processed.
    fn play_status(&mut self, playing: bool, forward: bool);
    /// Called when a coach joins the session.
    fn coach_joined(&mut self, coach: &str, all_coaches: &[String], replay_name: &str);
    /// Called when a coach leaves the session.
    fn coach_left(&mut self, coach: &str, all_coaches: &[String]);
    /// Called when the replay mode choice dialog should be shown.
    fn prompt_for_replay_choice(&mut self);
}

/// Java: `ReplayLogicModule`.
#[derive(Default)]
pub struct ReplayLogicModule {
    /// Java: `private List<ServerCommand> fReplayList`.
    replay_list: Option<Vec<ServerCommand>>,
    /// Java: `private Set<Integer> markingAffectingCommands`.
    marking_affecting_commands: HashSet<i32>,
    /// Java: `private boolean replayerInitialized`. Only ever flipped to `true` inside the
    /// `SERVER_REPLAY` branch of `handleCommand`, which is a documented no-op gap here (see
    /// module doc) — so this field is currently write-once-never (kept for field-for-field
    /// fidelity with the Java class).
    #[allow(dead_code)]
    replayer_initialized: bool,
    /// Java: `private ReplayCallbacks callbacks`.
    callbacks: Option<Box<dyn ReplayCallbacks>>,
}

impl ReplayLogicModule {
    /// Java: `public ReplayLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self {
            replay_list: None,
            marking_affecting_commands: HashSet::new(),
            replayer_initialized: false,
            callbacks: None,
        }
    }

    /// java: `public void setCallbacks(ReplayCallbacks callbacks)`
    pub fn set_callbacks(&mut self, callbacks: Box<dyn ReplayCallbacks>) {
        self.callbacks = Some(callbacks);
    }

    /// java: `private void startLoadingReplay(ClientReplayer replayer, ClientParameters parameters)`
    ///
    /// java: gap — `replayer.start()` has no translated target (see module doc).
    fn start_loading_replay(&self, client: &mut FantasyFootballClient) {
        let game_id = client.parameters().game_id();
        let coach = client.parameters().coach().unwrap_or_default().to_string();
        client.communication_mut().send_replay(game_id, 0, coach);
    }

    /// java: `public void handleCommand(NetCommand pNetCommand)` — see module doc for the
    /// documented gaps: only the `SERVER_GAME_STATE` case is fully translatable given the
    /// currently-modeled `ServerCommand` payloads.
    pub fn handle_command(&mut self, client: &mut FantasyFootballClient, net_command: &ServerCommand) {
        match net_command {
            // java: `case SERVER_GAME_STATE:`
            ServerCommand::ServerGameState(_) => {
                if client.mode() == Some(ClientMode::REPLAY) {
                    self.replay_list = Some(Vec::new());
                    if let Some(callbacks) = &mut self.callbacks {
                        callbacks.reset();
                    }
                }
            }
            // java: gap — `SERVER_USER_SETTINGS`, `SERVER_REPLAY`, `SERVER_STATUS`,
            // `SERVER_AUTOMATIC_PLAYER_MARKINGS`, `SERVER_REPLAY_STATUS`,
            // `SERVER_REPLAY_CONTROL`, `SERVER_JOIN`, `SERVER_LEAVE` all read payload fields
            // (`getMarkingAffectingCommands()`, `getReplayCommands()`, `getServerStatus()`,
            // `getMarkings()`, `getCoach()`/`getSpectators()`/`getReplayName()`, ...) that the
            // ported `ServerCommand` enum does not carry (see module doc) — left as a no-op.
            _ => {}
        }
    }

    /// java: `public void evaluateControl(boolean hasControl, String coach)`
    ///
    /// java: gap — `client.getReplayer().setControl(hasControl)` and the `getOverlays()`
    /// `ControlAware` notification loop have no translated targets (see module doc).
    pub fn evaluate_control(&mut self, coach: &str) {
        if let Some(callbacks) = &mut self.callbacks {
            callbacks.control_changed(coach);
        }
    }

    /// java: `private void updateClientData(List<String> allCoaches)`. Only called from the
    /// `SERVER_JOIN`/`SERVER_LEAVE` branches of `handleCommand`, both documented no-op gaps
    /// here (see module doc) — kept for field-for-field fidelity with the Java class.
    #[allow(dead_code)]
    fn update_client_data(&self, client: &mut FantasyFootballClient, all_coaches: &[String]) {
        let own_coach = client.parameters().coach().unwrap_or_default().to_string();
        let filtered_coaches: Vec<String> =
            all_coaches.iter().filter(|coach| **coach != own_coach).cloned().collect();
        let count = filtered_coaches.len() as i32;
        client.client_data_mut().set_spectator_count(count);
        client.client_data_mut().set_spectators(filtered_coaches);
    }

    /// java: `public void replayMode(boolean online, String name)`
    ///
    /// java: gap — `client.getReplayer().setOnline(true)`, the `getOverlays()` `OnlineAware`
    /// notification loop, and `client.replayInitialized()` have no translated targets (see
    /// module doc).
    pub fn replay_mode(&mut self, client: &mut FantasyFootballClient, online: bool, name: &str) {
        let sanitized_name: String = name.chars().take(REPLAY_NAME_MAX_LENGTH).collect();
        let coach = client.parameters().coach().unwrap_or_default().to_string();
        if online && string_tool::is_provided(Some(sanitized_name.as_str())) {
            let game_id = client.parameters().game_id();
            client.communication_mut().send_join_replay(sanitized_name, coach, game_id);
        } else {
            client.communication_mut().send_close_session();
            self.evaluate_control(&coach);
        }
    }

    /// java: `public boolean replayStopped(ActionKey pActionKey)`
    pub fn replay_stopped(&self, client: &FantasyFootballClient, action_key: ActionKey) -> bool {
        client.mode() == Some(ClientMode::SPECTATOR) && action_key == ActionKey::MENU_REPLAY
    }

    /// java: `public boolean isOnline()` — java: gap, `client.getReplayer().isOnline()` has no
    /// translated target (see module doc); conservatively `false`.
    pub fn is_online(&self, _client: &FantasyFootballClient) -> bool {
        false
    }
}

impl LogicModule for ReplayLogicModule {
    /// java: `public ClientStateId getId()`
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Replay
    }

    /// java: `public void setUp()` — see module doc for the `getReplayer()`/`getProperty()`
    /// documented gaps; the reachable, translatable parts (choosing between joining an
    /// authenticated session vs. starting a fresh replay load) are implemented.
    fn set_up(&mut self, client: &mut FantasyFootballClient) {
        self.marking_affecting_commands = HashSet::new();
        // java: gap — `IClientPropertyValue.AUTO_MARKING.contains(client.getProperty(...))`
        // seeds `markingAffectingCommands` with `-1`; `getProperty()` has no in-scope body.

        if client.mode() == Some(ClientMode::REPLAY) {
            // java: gap — `replayer.setControl(false)`.
            let authentication = client.parameters().authentication().map(|s| s.to_string());
            if string_tool::is_provided(authentication.as_deref()) {
                let coach = client.parameters().coach().unwrap_or_default().to_string();
                let auth = authentication.unwrap_or_default();
                let client_mode = client.mode().unwrap_or(ClientMode::REPLAY);
                client.communication_mut().send_join(client_mode, coach, auth, 0, "", "", "");
            } else {
                self.start_loading_replay(client);
            }
        } else if self.replay_list.is_none() {
            self.replay_list = Some(Vec::new());
            if let Some(callbacks) = &mut self.callbacks {
                callbacks.reset();
            }
            let coach = client.parameters().coach().unwrap_or_default().to_string();
            // java: `replayer.getFirstCommandNr()` — java: gap, no translated target; `0` used
            // as the conservative starting point (matches a fresh, not-yet-positioned replayer).
            client.communication_mut().send_replay(0, 0, coach);
        }
        // java: gap — the `else` branch's `replayer.positionOnLastCommand()` /
        // `getReplayControl().setActive(true)` have no translated targets.
    }

    /// java: `public Set<ClientAction> availableActions()`
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in replay context");
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// empty body in Java.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_parameters::ClientParameters;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use ffb_protocol::server_commands::ServerGameState;

    #[derive(Default)]
    struct RecordingCallbacks {
        reset_calls: i32,
        control_changed: Option<String>,
    }

    impl ReplayCallbacks for RecordingCallbacks {
        fn reset(&mut self) {
            self.reset_calls += 1;
        }
        fn command_count(&mut self, _total_commands: i32) {}
        fn loaded_commands(&mut self, _current_size: i32) {}
        fn load_done(&mut self) {}
        fn start_replayer_init(&mut self) {}
        fn replayer_initialized(&mut self) {}
        fn progress_listener(&mut self) -> Box<dyn IProgressListener> {
            struct NoOp;
            impl IProgressListener for NoOp {
                fn init_progress(&mut self, _minimum: i32, _maximum: i32) {}
                fn update_progress(&mut self, _progress: i32) {}
            }
            Box::new(NoOp)
        }
        fn replay_unavailable(&mut self, _status: ffb_model::enums::ServerStatus) {}
        fn control_changed(&mut self, controlling_coach: &str) {
            self.control_changed = Some(controlling_coach.to_string());
        }
        fn play_status(&mut self, _playing: bool, _forward: bool) {}
        fn coach_joined(&mut self, _coach: &str, _all_coaches: &[String], _replay_name: &str) {}
        fn coach_left(&mut self, _coach: &str, _all_coaches: &[String]) {}
        fn prompt_for_replay_choice(&mut self) {}
    }

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

    fn make_client(mode: ClientMode) -> FantasyFootballClient {
        let args: Vec<String> = match mode {
            ClientMode::PLAYER => vec!["-player".into(), "-coach".into(), "bob".into()],
            ClientMode::SPECTATOR => vec!["-spectator".into(), "-coach".into(), "bob".into()],
            ClientMode::REPLAY => vec!["-replay".into(), "-gameId".into(), "1".into()],
        };
        let params = ClientParameters::create_valid_params(&args).unwrap();
        let mut client = FantasyFootballClient::new(params);
        client.set_game(Game::new(make_team("home"), make_team("away"), Rules::Bb2025));
        client
    }

    #[test]
    fn get_id_is_replay() {
        let module = ReplayLogicModule::new();
        assert_eq!(module.get_id(), ClientStateId::Replay);
    }

    #[test]
    fn available_actions_is_always_empty() {
        let module = ReplayLogicModule::new();
        assert!(module.available_actions().is_empty());
    }

    #[test]
    fn set_up_initializes_replay_list_in_non_replay_mode() {
        let mut module = ReplayLogicModule::new();
        module.set_callbacks(Box::new(RecordingCallbacks::default()));
        let mut client = make_client(ClientMode::SPECTATOR);
        module.set_up(&mut client);
        assert!(module.replay_list.is_some());
    }

    #[test]
    fn set_up_does_not_reinitialize_existing_replay_list() {
        let mut module = ReplayLogicModule::new();
        let mut client = make_client(ClientMode::SPECTATOR);
        module.replay_list = Some(vec![ServerCommand::ServerGameState(ServerGameState {
            command_nr: 1,
            game: Box::new(Game::new(make_team("home"), make_team("away"), Rules::Bb2025)),
        })]);
        module.set_up(&mut client);
        assert_eq!(module.replay_list.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn handle_command_resets_replay_list_on_game_state_in_replay_mode() {
        let mut module = ReplayLogicModule::new();
        module.set_callbacks(Box::new(RecordingCallbacks::default()));
        let mut client = make_client(ClientMode::REPLAY);
        module.replay_list = Some(vec![ServerCommand::ServerGameState(ServerGameState {
            command_nr: 1,
            game: Box::new(Game::new(make_team("home"), make_team("away"), Rules::Bb2025)),
        })]);
        let cmd = ServerCommand::ServerGameState(ServerGameState {
            command_nr: 2,
            game: Box::new(Game::new(make_team("home"), make_team("away"), Rules::Bb2025)),
        });
        module.handle_command(&mut client, &cmd);
        assert_eq!(module.replay_list.as_ref().unwrap().len(), 0);
    }

    #[test]
    fn handle_command_ignores_game_state_when_not_replay_mode() {
        let mut module = ReplayLogicModule::new();
        let mut client = make_client(ClientMode::SPECTATOR);
        module.replay_list = Some(vec![ServerCommand::ServerGameState(ServerGameState {
            command_nr: 1,
            game: Box::new(Game::new(make_team("home"), make_team("away"), Rules::Bb2025)),
        })]);
        let cmd = ServerCommand::ServerGameState(ServerGameState {
            command_nr: 2,
            game: Box::new(Game::new(make_team("home"), make_team("away"), Rules::Bb2025)),
        });
        module.handle_command(&mut client, &cmd);
        assert_eq!(module.replay_list.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn evaluate_control_notifies_callbacks() {
        let mut module = ReplayLogicModule::new();
        module.set_callbacks(Box::new(RecordingCallbacks::default()));
        module.evaluate_control("coach1");
        // Can't downcast the trait object back in this simple harness; exercised for panics only.
    }

    #[test]
    fn replay_stopped_true_only_for_spectator_mode_and_menu_replay_key() {
        let module = ReplayLogicModule::new();
        let spectator = make_client(ClientMode::SPECTATOR);
        let player = make_client(ClientMode::PLAYER);
        assert!(module.replay_stopped(&spectator, ActionKey::MENU_REPLAY));
        assert!(!module.replay_stopped(&spectator, ActionKey::TOOLBAR_TURN_END));
        assert!(!module.replay_stopped(&player, ActionKey::MENU_REPLAY));
    }

    #[test]
    fn is_online_is_always_false_gap() {
        let module = ReplayLogicModule::new();
        let client = make_client(ClientMode::SPECTATOR);
        assert!(!module.is_online(&client));
    }

    #[test]
    fn replay_mode_offline_closes_session_and_evaluates_control() {
        let mut module = ReplayLogicModule::new();
        module.set_callbacks(Box::new(RecordingCallbacks::default()));
        let mut client = make_client(ClientMode::SPECTATOR);
        module.replay_mode(&mut client, false, "MyReplay");
        // No panics; communication outbox should have a close-session command queued.
        assert!(!client.communication().outbox.is_empty());
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in replay context")]
    fn action_context_panics() {
        let module = ReplayLogicModule::new();
        let client = make_client(ClientMode::SPECTATOR);
        let game = client.game().unwrap();
        let ap = ActingPlayer::new();
        module.action_context(game, &ap);
    }
}
