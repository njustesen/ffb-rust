/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerPing.
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ffb_model::enums::NetCommandId;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerPing`.
pub struct ServerCommandHandlerPing {
    session_manager: Arc<Mutex<SessionManager>>,
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerPing {
    /// Java: `protected ServerCommandHandlerPing(FantasyFootballServer pServer)`.
    pub fn new(
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self {
            session_manager,
            replay_session_manager,
        }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_PING`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPing
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// `ping_timestamp` is `ClientCommandPing.getTimestamp()` — the client-supplied timestamp
    /// echoed back in the pong.
    pub fn handle_command(&self, session_id: SessionId, ping_timestamp: i64) -> bool {
        let now = current_time_millis();

        if self.replay_session_manager.lock().unwrap().has(session_id) {
            self.replay_session_manager
                .lock()
                .unwrap()
                .set_last_ping(session_id, now);
        } else {
            self.session_manager
                .lock()
                .unwrap()
                .set_last_ping(session_id, now);
        }

        // Java: `getServer().getCommunication().sendPong(session, pingCommand.getTimestamp())`.
        let pong = serde_json::json!({
            "netCommandId": "serverPong",
            "timestamp": ping_timestamp,
        });
        self.session_manager
            .lock()
            .unwrap()
            .send_to(session_id, &pong.to_string());

        true
    }
}

fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<ReplaySessionManager>>,
        mpsc::UnboundedReceiver<String>,
    ) {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let (tx, rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(1, 100, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        (sm, rsm, rx)
    }

    #[test]
    fn get_id_is_client_ping() {
        let (sm, rsm, _rx) = setup();
        let handler = ServerCommandHandlerPing::new(sm, rsm);
        assert_eq!(handler.get_id(), NetCommandId::ClientPing);
    }

    #[test]
    fn handle_command_updates_last_ping_and_returns_true() {
        let (sm, rsm, _rx) = setup();
        let handler = ServerCommandHandlerPing::new(Arc::clone(&sm), rsm);
        let before = sm.lock().unwrap().get_last_ping(1);
        assert_eq!(before, 0);

        let ok = handler.handle_command(1, 4242);
        assert!(ok);
        assert!(sm.lock().unwrap().get_last_ping(1) > 0);
    }

    #[test]
    fn handle_command_sends_pong_with_echoed_timestamp() {
        let (sm, rsm, mut rx) = setup();
        let handler = ServerCommandHandlerPing::new(sm, rsm);

        handler.handle_command(1, 9999);

        let msg = rx.try_recv().expect("expected a pong message");
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["netCommandId"], "serverPong");
        assert_eq!(value["timestamp"], 9999);
    }

    #[test]
    fn handle_command_updates_replay_last_ping_when_replay_session() {
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let mut replay = ReplaySessionManager::new();
        replay.add_session(7, "replay1".into(), "Coach".into());
        let rsm = Arc::new(Mutex::new(replay));
        let handler = ServerCommandHandlerPing::new(sm, Arc::clone(&rsm));

        let ok = handler.handle_command(7, 1);
        assert!(ok);
        assert!(rsm.lock().unwrap().get_last_ping(7) > 0);
    }
}
