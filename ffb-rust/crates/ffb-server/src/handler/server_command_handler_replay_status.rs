/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerReplayStatus.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ffb_engine::replay_state::ReplayState;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_replay_status::ClientCommandReplayStatus;
use ffb_protocol::commands::server_command_replay_status::ServerCommandReplayStatus;
use crate::net::replay_session_manager::ReplaySessionManager;

/// Java: `ServerCommandHandlerReplayStatus extends ServerCommandHandler`.
///
/// Java's `getServer().getReplayCache().replayState(name)` returns a `ReplayState`
/// keyed by replay name; the Rust MVP has no server-level `ReplayCache` wiring yet, so
/// (matching `ServerCommandHandlerTransferControl` / `ServerCommandHandlerSetPreventSketching`)
/// this handler owns its own name -> `ReplayState` map directly.
pub struct ServerCommandHandlerReplayStatus {
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
}

impl ServerCommandHandlerReplayStatus {
    pub fn new(
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
    ) -> Self {
        Self { replay_session_manager, replay_states }
    }

    /// Java: getId() — returns NetCommandId for REPLAY_STATUS.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientReplayStatus
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles a replay status update.
    ///
    /// Only the controlling session's status updates are pushed to the other
    /// sessions sharing the replay, and only when they actually change
    /// playback state (per the cached `ReplayState`, read *before* Java's
    /// `ServerCommunication.handleByReplayState` updates it from this same
    /// incoming command — that update happens in `ServerCommunication`'s
    /// dispatch loop, after this handler runs, and is out of scope here).
    pub fn handle_command(&self, cmd: &ClientCommandReplayStatus, session_id: u64) -> bool {
        let (has_control, replay_name) = {
            let sm = self.replay_session_manager.lock().unwrap();
            (sm.has_control(session_id), sm.replay_name_for_session(session_id))
        };

        if has_control && !replay_name.is_empty() {
            // Java: ReplayCache cache = getServer().getReplayCache();
            //       ReplayState state = cache.replayState(replayName);
            //       if (state != null && requiresPushToOtherClients(state, cmd)) { ... }
            let requires_push = {
                let states = self.replay_states.lock().unwrap();
                states.get(&replay_name).map(|state| {
                    Self::requires_push_to_other_clients(
                        state.is_forward(),
                        state.is_running(),
                        state.get_speed(),
                        cmd,
                    )
                })
            };

            if requires_push == Some(true) {
                let server_command = Self::to_server_command(cmd);
                let json = Self::to_json(&server_command);
                let others = { self.replay_session_manager.lock().unwrap().other_sessions(session_id) };
                let rsm = self.replay_session_manager.lock().unwrap();
                for other_session in others {
                    // Java: `getServer().getCommunication().send(otherSession, serverCommandReplayStatus, true)`.
                    rsm.send_to(other_session, &json);
                }
            }
        }

        true
    }

    /// Java: `requiresPushToOtherClients(ReplayState, ClientCommandReplayStatus)`.
    fn requires_push_to_other_clients(
        state_forward: bool,
        state_running: bool,
        state_speed: i32,
        cmd: &ClientCommandReplayStatus,
    ) -> bool {
        cmd.is_skip()
            || cmd.is_forward() != state_forward
            || cmd.is_running() != state_running
            || cmd.get_speed() != state_speed
    }

    /// Java: `new ServerCommandReplayStatus(commandNr, speed, running, forward, skip)`.
    fn to_server_command(cmd: &ClientCommandReplayStatus) -> ServerCommandReplayStatus {
        ServerCommandReplayStatus::new(cmd.get_command_nr(), cmd.get_speed(), cmd.is_running(), cmd.is_forward(), cmd.is_skip())
    }

    /// `ServerCommandReplayStatus` has no serde impl of its own (it isn't a
    /// `ffb_protocol::server_commands::ServerCommand` variant), so the JSON is built directly
    /// from its fields, same as the sketch handlers' manual `to_json` helpers.
    fn to_json(cmd: &ServerCommandReplayStatus) -> String {
        format!(
            "{{\"netCommandId\":\"serverReplayStatus\",\"commandNr\":{},\"speed\":{},\"running\":{},\"forward\":{},\"skip\":{}}}",
            cmd.get_command_nr(),
            cmd.get_speed(),
            cmd.is_running(),
            cmd.is_forward(),
            cmd.is_skip()
        )
    }
}

impl Default for ServerCommandHandlerReplayStatus {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(ReplaySessionManager::new())), Arc::new(Mutex::new(HashMap::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerReplayStatus::default();
    }

    #[test]
    fn get_id_returns_client_replay_status() {
        let h = ServerCommandHandlerReplayStatus::default();
        assert_eq!(h.get_id(), NetCommandId::ClientReplayStatus);
    }

    #[test]
    fn handle_command_without_control_is_noop() {
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        {
            let mut m = rsm.lock().unwrap();
            m.add_session(1, "replay1".to_string(), "coach1".to_string());
            m.add_session(2, "replay1".to_string(), "coach2".to_string());
        }
        let h = ServerCommandHandlerReplayStatus::new(rsm, Arc::new(Mutex::new(HashMap::new())));
        let cmd = ClientCommandReplayStatus::with_params(10, 1, true, true, false);
        // Session 2 has no control, so the ReplayCache lookup is never reached.
        assert!(h.handle_command(&cmd, 2));
    }

    #[test]
    fn handle_command_with_control_but_no_cached_state_is_noop() {
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        {
            let mut m = rsm.lock().unwrap();
            m.add_session(1, "replay1".to_string(), "coach1".to_string());
        }
        let h = ServerCommandHandlerReplayStatus::new(rsm, Arc::new(Mutex::new(HashMap::new())));
        let cmd = ClientCommandReplayStatus::with_params(10, 1, true, true, false);
        // No ReplayState cached yet for "replay1" — nothing to compare against.
        assert!(h.handle_command(&cmd, 1));
    }

    #[test]
    fn handle_command_with_control_and_changed_state_broadcasts_to_others() {
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut m = rsm.lock().unwrap();
            m.add_session(1, "replay1".to_string(), "coach1".to_string());
            m.add_session(2, "replay1".to_string(), "coach2".to_string());
            m.register_sender(2, tx2);
        }
        let states = Arc::new(Mutex::new(HashMap::new()));
        states.lock().unwrap().insert("replay1".to_string(), ReplayState::new("replay1"));
        let h = ServerCommandHandlerReplayStatus::new(rsm, states);
        // Cached state defaults to speed=0/running=false/forward=false; this command differs.
        let cmd = ClientCommandReplayStatus::with_params(10, 1, true, true, false);
        assert!(h.handle_command(&cmd, 1));
        let sent = rx2.try_recv().expect("expected a broadcast to session 2");
        assert!(sent.contains("serverReplayStatus"));
        assert!(sent.contains("\"commandNr\":10"));
    }

    #[test]
    fn handle_command_with_control_and_unchanged_state_is_noop() {
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut m = rsm.lock().unwrap();
            m.add_session(1, "replay1".to_string(), "coach1".to_string());
            m.add_session(2, "replay1".to_string(), "coach2".to_string());
            m.register_sender(2, tx2.clone());
        }
        let states = Arc::new(Mutex::new(HashMap::new()));
        states.lock().unwrap().insert("replay1".to_string(), ReplayState::new("replay1"));
        let h = ServerCommandHandlerReplayStatus::new(rsm, states);
        // Cached state defaults to speed=0/running=false/forward=false, not skip — this
        // command matches exactly, so no push is required.
        let cmd = ClientCommandReplayStatus::with_params(0, 0, false, false, false);
        assert!(h.handle_command(&cmd, 1));
        assert!(rx2.try_recv().is_err());
        drop(tx2);
    }

    #[test]
    fn requires_push_when_skip_flag_set() {
        let cmd = ClientCommandReplayStatus::with_params(1, 1, true, true, true);
        assert!(ServerCommandHandlerReplayStatus::requires_push_to_other_clients(true, true, 1, &cmd));
    }

    #[test]
    fn requires_push_when_state_matches_and_not_skip() {
        let cmd = ClientCommandReplayStatus::with_params(1, 2, true, true, false);
        assert!(!ServerCommandHandlerReplayStatus::requires_push_to_other_clients(true, true, 2, &cmd));
    }

    #[test]
    fn requires_push_when_speed_differs() {
        let cmd = ClientCommandReplayStatus::with_params(1, 3, true, true, false);
        assert!(ServerCommandHandlerReplayStatus::requires_push_to_other_clients(true, true, 1, &cmd));
    }

    #[test]
    fn to_server_command_preserves_fields() {
        let cmd = ClientCommandReplayStatus::with_params(42, 2, true, false, true);
        let server_cmd = ServerCommandHandlerReplayStatus::to_server_command(&cmd);
        assert_eq!(server_cmd.get_command_nr(), 42);
        assert_eq!(server_cmd.get_speed(), 2);
        assert!(server_cmd.is_running());
        assert!(!server_cmd.is_forward());
        assert!(server_cmd.is_skip());
    }
}
