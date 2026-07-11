/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerTransferControl.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ffb_engine::replay_state::ReplayState;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_transfer_replay_control::ClientCommandTransferReplayControl;
use ffb_protocol::commands::server_command_replay_control::ServerCommandReplayControl;
use ffb_protocol::commands::server_command_set_prevent_sketching::ServerCommandSetPreventSketching;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerTransferControl extends ServerCommandHandler`.
///
/// Java's `getServer().getReplayCache().replayState(name)` returns a
/// `ReplayState` keyed by replay name; the Rust MVP has no server-level
/// `ReplayCache` wiring, so this handler owns its own name → `ReplayState`
/// map directly (the `ffb_engine::replay_state::ReplayState` 1:1 translation
/// already tracks per-coach sketching-prevention).
pub struct ServerCommandHandlerTransferControl {
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerTransferControl {
    /// Java: `protected ServerCommandHandlerTransferControl(FantasyFootballServer pServer)`
    pub fn new(
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> Self {
        Self { replay_session_manager, replay_states, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_TRANSFER_REPLAY_CONTROL`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTransferReplayControl
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandTransferReplayControl,
    ) -> bool {
        if let Some(coach) = command.get_coach() {
            // Java: `StringTool.isProvided(command.getCoach())`.
            if !coach.is_empty() {
                let transferred = self
                    .replay_session_manager
                    .lock()
                    .unwrap()
                    .transfer_control(session_id, coach);

                if transferred {
                    let replay_name = self
                        .replay_session_manager
                        .lock()
                        .unwrap()
                        .replay_name_for_session(session_id);

                    let mut states = self.replay_states.lock().unwrap();
                    if let Some(replay_state) = states.get_mut(&replay_name) {
                        let rsm = self.replay_session_manager.lock().unwrap();
                        let sessions = rsm.sessions_for_replay(&replay_name).unwrap_or_default();
                        let sm = self.session_manager.lock().unwrap();

                        // Java: `getServer().getCommunication().sendReplayControlChange(replayState, coach)`.
                        let control_msg = ServerCommandReplayControl::new(coach);
                        let json = replay_control_to_json(&control_msg);
                        for &s in &sessions {
                            sm.send_to(s, &json);
                        }

                        if replay_state.is_coach_prevented_from_sketching(coach) {
                            // Java: `getServer().getCommunication().sendReplayPreventSketching(replayState, coach, false)`.
                            let sketch_msg = ServerCommandSetPreventSketching::new(coach, false);
                            let json = prevent_sketching_to_json(&sketch_msg);
                            for &s in &sessions {
                                sm.send_to(s, &json);
                            }
                        }
                    }
                }
            }
        }
        true
    }
}

/// Manual JSON encoding for `ServerCommandReplayControl` (see `talk_to_json`
/// in `server_command_handler_talk.rs` for why: the `ffb_protocol::commands`
/// structs don't derive `Serialize`).
fn replay_control_to_json(msg: &ServerCommandReplayControl) -> String {
    serde_json::json!({
        "netCommandId": "serverReplayControl",
        "coach": msg.get_coach(),
    })
    .to_string()
}

/// Manual JSON encoding for `ServerCommandSetPreventSketching`.
fn prevent_sketching_to_json(msg: &ServerCommandSetPreventSketching) -> String {
    serde_json::json!({
        "netCommandId": "serverSetPreventSketching",
        "preventSketching": msg.is_prevent_sketching(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (
        Arc<Mutex<ReplaySessionManager>>,
        Arc<Mutex<HashMap<String, ReplayState>>>,
        Arc<Mutex<SessionManager>>,
    ) {
        (
            Arc::new(Mutex::new(ReplaySessionManager::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(SessionManager::new())),
        )
    }

    #[test]
    fn construct() {
        let (rsm, states, sm) = setup();
        let _ = ServerCommandHandlerTransferControl::new(rsm, states, sm);
    }

    #[test]
    fn get_id_is_client_transfer_replay_control() {
        let (rsm, states, sm) = setup();
        let handler = ServerCommandHandlerTransferControl::new(rsm, states, sm);
        assert_eq!(handler.get_id(), NetCommandId::ClientTransferReplayControl);
    }

    #[test]
    fn missing_coach_is_a_noop() {
        let (rsm, states, sm) = setup();
        let handler = ServerCommandHandlerTransferControl::new(rsm, states, sm);
        let result = handler.handle_command(1, &ClientCommandTransferReplayControl::new());
        assert!(result);
    }

    #[test]
    fn successful_transfer_broadcasts_control_change() {
        let (rsm, states, sm) = setup();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut r = rsm.lock().unwrap();
            r.add_session(1, "replay1".into(), "coach1".into());
            r.add_session(2, "replay1".into(), "coach2".into());
            let mut s = sm.lock().unwrap();
            s.add_session(1, 0, "coach1".into(), ClientMode::SPECTATOR, false, vec![], tx1);
            s.add_session(2, 0, "coach2".into(), ClientMode::SPECTATOR, false, vec![], tx2);
            states.lock().unwrap().insert("replay1".to_string(), ReplayState::new("replay1"));
        }

        let handler = ServerCommandHandlerTransferControl::new(rsm.clone(), states, sm);
        let result = handler.handle_command(1, &ClientCommandTransferReplayControl::with_coach("coach2"));
        assert!(result);
        assert!(rsm.lock().unwrap().has_control(2));

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();
        assert!(msg1.contains("coach2"));
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn transfer_to_prevented_coach_also_sends_allow_sketching() {
        let (rsm, states, sm) = setup();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();
        {
            let mut r = rsm.lock().unwrap();
            r.add_session(1, "replay1".into(), "coach1".into());
            r.add_session(2, "replay1".into(), "coach2".into());
            let mut s = sm.lock().unwrap();
            s.add_session(1, 0, "coach1".into(), ClientMode::SPECTATOR, false, vec![], tx1);
            s.add_session(2, 0, "coach2".into(), ClientMode::SPECTATOR, false, vec![], tx2);
            let mut rs = ReplayState::new("replay1");
            rs.prevent_coach_from_sketching("coach2");
            states.lock().unwrap().insert("replay1".to_string(), rs);
        }

        let handler = ServerCommandHandlerTransferControl::new(rsm, states, sm);
        handler.handle_command(1, &ClientCommandTransferReplayControl::with_coach("coach2"));

        // First message is the control-change broadcast, second is the prevent-sketching one.
        let _control_msg = rx1.try_recv().unwrap();
        let sketch_msg = rx1.try_recv().unwrap();
        assert!(sketch_msg.contains("preventSketching") || sketch_msg.contains("prevent_sketching"));
    }

    #[test]
    fn transfer_failure_sends_nothing() {
        let (rsm, states, sm) = setup();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        {
            let mut r = rsm.lock().unwrap();
            r.add_session(1, "replay1".into(), "coach1".into());
            let mut s = sm.lock().unwrap();
            s.add_session(1, 0, "coach1".into(), ClientMode::SPECTATOR, false, vec![], tx1.clone());
        }
        let handler = ServerCommandHandlerTransferControl::new(rsm, states, sm);
        // Target coach not present in the replay — transfer_control fails.
        let result = handler.handle_command(1, &ClientCommandTransferReplayControl::with_coach("nobody"));
        assert!(result);
        assert!(rx1.try_recv().is_err());
        drop(tx1);
    }
}
