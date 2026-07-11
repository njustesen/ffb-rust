/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerSetPreventSketching.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ffb_engine::replay_state::ReplayState;
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_set_prevent_sketching::ClientCommandSetPreventSketching;
use ffb_protocol::commands::server_command_set_prevent_sketching::ServerCommandSetPreventSketching;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerSetPreventSketching extends ServerCommandHandler`.
///
/// As in `ServerCommandHandlerTransferControl`, the Rust MVP has no
/// server-level `ReplayCache`, so this handler owns its own replay-name →
/// `ReplayState` map directly.
pub struct ServerCommandHandlerSetPreventSketching {
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerSetPreventSketching {
    /// Java: `protected ServerCommandHandlerSetPreventSketching(FantasyFootballServer pServer)`
    pub fn new(
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        replay_states: Arc<Mutex<HashMap<String, ReplayState>>>,
        session_manager: Arc<Mutex<SessionManager>>,
    ) -> Self {
        Self { replay_session_manager, replay_states, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_SET_PREVENT_SKETCHING`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetPreventSketching
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(
        &self,
        session_id: SessionId,
        command: &ClientCommandSetPreventSketching,
    ) -> bool {
        let rsm = self.replay_session_manager.lock().unwrap();
        if rsm.has(session_id) && rsm.has_control(session_id) {
            let replay_name = rsm.replay_for_session(session_id);
            // Java: `command.getCoach()` may be null; a `HashSet<String>` tolerates that,
            // but `ReplayState`'s coach set here is keyed by non-optional `&str`.
            let coach = command.get_coach().unwrap_or("");

            {
                let mut states = self.replay_states.lock().unwrap();
                if let Some(replay_state) = states.get_mut(&replay_name) {
                    // Java: `synchronized (replayState) { ... }` — the surrounding mutex lock
                    // on `replay_states` already serializes access here.
                    if command.is_prevent_sketching() {
                        replay_state.prevent_coach_from_sketching(coach);
                    } else {
                        replay_state.allow_coach_to_sketch(coach);
                    }
                }
            }

            // Java: `Set<Session> sessions = new HashSet<>(); sessions.add(session);
            // sessions.addAll(sessionManager.otherSessions(session));`
            let mut sessions: Vec<SessionId> = vec![session_id];
            sessions.extend(rsm.other_sessions(session_id));

            // Java: `new ServerCommandSetPreventSketching(command.getCoach(), command.isPreventSketching())`
            let msg = ServerCommandSetPreventSketching::new(
                command.get_coach().unwrap_or("").to_string(),
                command.is_prevent_sketching(),
            );
            let json = prevent_sketching_to_json(&msg);
            let sm = self.session_manager.lock().unwrap();
            for s in sessions {
                sm.send_to(s, &json);
            }
        }
        true
    }
}

/// Manual JSON encoding for `ServerCommandSetPreventSketching` — the
/// `ffb_protocol::commands` structs don't derive `Serialize`.
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
        let _ = ServerCommandHandlerSetPreventSketching::new(rsm, states, sm);
    }

    #[test]
    fn get_id_is_client_set_prevent_sketching() {
        let (rsm, states, sm) = setup();
        let handler = ServerCommandHandlerSetPreventSketching::new(rsm, states, sm);
        assert_eq!(handler.get_id(), NetCommandId::ClientSetPreventSketching);
    }

    #[test]
    fn non_replay_session_is_a_noop() {
        let (rsm, states, sm) = setup();
        let handler = ServerCommandHandlerSetPreventSketching::new(rsm, states, sm);
        let result = handler.handle_command(1, &ClientCommandSetPreventSketching::with_fields("coach1", true));
        assert!(result);
    }

    #[test]
    fn session_without_control_is_a_noop() {
        let (rsm, states, sm) = setup();
        rsm.lock().unwrap().add_session(1, "replay1".into(), "coach1".into());
        rsm.lock().unwrap().add_session(2, "replay1".into(), "coach2".into()); // controlling session
        let handler = ServerCommandHandlerSetPreventSketching::new(rsm, states, sm);
        // Session 1 joined first and would have control unless another session took it;
        // use session 2, which never has control since session 1 got it first.
        let result = handler.handle_command(2, &ClientCommandSetPreventSketching::with_fields("coach1", true));
        assert!(result);
    }

    #[test]
    fn controlling_session_prevents_sketching_and_broadcasts() {
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

        let handler = ServerCommandHandlerSetPreventSketching::new(rsm, states.clone(), sm);
        let result = handler.handle_command(1, &ClientCommandSetPreventSketching::with_fields("coach2", true));
        assert!(result);

        let states = states.lock().unwrap();
        let rs = states.get("replay1").unwrap();
        assert!(rs.is_coach_prevented_from_sketching("coach2"));

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn allow_sketching_clears_prevention() {
        let (rsm, states, sm) = setup();
        {
            let mut r = rsm.lock().unwrap();
            r.add_session(1, "replay1".into(), "coach1".into());
            let mut rs = ReplayState::new("replay1");
            rs.prevent_coach_from_sketching("coach1");
            states.lock().unwrap().insert("replay1".to_string(), rs);
        }

        let handler = ServerCommandHandlerSetPreventSketching::new(rsm, states.clone(), sm);
        handler.handle_command(1, &ClientCommandSetPreventSketching::with_fields("coach1", false));

        let states = states.lock().unwrap();
        assert!(!states.get("replay1").unwrap().is_coach_prevented_from_sketching("coach1"));
    }
}
