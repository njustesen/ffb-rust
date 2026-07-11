/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerSocketClosed.
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_engine::util::util_server_timer::UtilServerTimer;
use ffb_model::enums::NetCommandId;
use ffb_model::model::ClientMode;

use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerSocketClosed`.
pub struct ServerCommandHandlerSocketClosed {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
}

impl ServerCommandHandlerSocketClosed {
    /// Java: `protected ServerCommandHandlerSocketClosed(FantasyFootballServer pServer)`.
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
    ) -> Self {
        Self {
            game_cache,
            session_manager,
            replay_session_manager,
            sketch_manager,
        }
    }

    /// Java: `getId()` — returns `NetCommandId.INTERNAL_SERVER_SOCKET_CLOSED`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerSocketClosed
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(&self, session_id: SessionId) -> bool {
        self.sketch_manager
            .lock()
            .unwrap()
            .remove_session(&session_id.to_string());

        let is_replay = self.replay_session_manager.lock().unwrap().has(session_id);
        if is_replay {
            self.close_replay_session(session_id)
        } else {
            self.close_game_session(session_id)
        }
    }

    /// Java: `closeGameSession(ReceivedCommand)`.
    pub fn close_game_session(&self, session_id: SessionId) -> bool {
        let (coach, mode, game_id, is_admin) = {
            let sm = self.session_manager.lock().unwrap();
            (
                sm.get_coach_for_session(session_id).map(|c| c.to_string()),
                sm.get_mode_for_session(session_id),
                sm.get_game_id_for_session(session_id),
                sm.is_session_admin(session_id),
            )
        };

        self.session_manager.lock().unwrap().remove_session(session_id);

        let sessions = self
            .session_manager
            .lock()
            .unwrap()
            .get_sessions_for_game_id(game_id);

        let game_exists = self
            .game_cache
            .lock()
            .unwrap()
            .get_game_state_by_id(game_id)
            .is_some();

        if game_exists {
            let mut spectators = Vec::new();
            {
                let sm = self.session_manager.lock().unwrap();
                for &s in &sessions {
                    if sm.get_mode_for_session(s) == Some(ClientMode::SPECTATOR)
                        && !sm.is_session_admin(s)
                    {
                        if let Some(c) = sm.get_coach_for_session(s) {
                            spectators.push(c.to_string());
                        }
                    }
                }
            }

            // Java: "stop timer whenever a player drops out".
            if mode == Some(ClientMode::PLAYER) {
                let now = current_time_millis();
                UtilServerTimer::sync_time(now);
                UtilServerTimer::stop_turn_timer(now);
            }

            // Java: if the game was ACTIVE and either coach seat is now empty, pause the game
            // and `gameCache.queueDbUpdate(gameState, true)`. `crate::game_state::GameState`
            // has no `status` field and `crate::game_cache::GameCache` has no DB-update queue
            // yet (Phase ZV, both outside handler/*.rs), so that status transition and its
            // persistence are not modeled here.

            if !sessions.is_empty() {
                let hide_leave_command = mode == Some(ClientMode::SPECTATOR) && is_admin;
                if !hide_leave_command {
                    // Java: `getServer().getCommunication().sendLeave(sessions, coach, mode, spectators)`.
                    let leave_msg = serde_json::json!({
                        "netCommandId": "serverLeave",
                        "coach": coach,
                        "mode": mode,
                        "spectators": spectators,
                    });
                    let text = leave_msg.to_string();
                    let sm = self.session_manager.lock().unwrap();
                    for &s in &sessions {
                        sm.send_to(s, &text);
                    }
                }
            } else {
                // Java: `getServer().getGameCache().closeGame(gameState.getId())`.
                // `crate::game_cache::GameCache` has no removal API yet (Phase ZV, outside
                // handler/*.rs), so the now-empty game is intentionally left cached.
            }
        }

        true
    }

    /// Java: `closeReplaySession(ReceivedCommand)`.
    pub fn close_replay_session(&self, session_id: SessionId) -> bool {
        let mut rsm = self.replay_session_manager.lock().unwrap();

        if rsm.has(session_id) {
            let other_sessions = rsm.other_sessions(session_id);

            if !other_sessions.is_empty() {
                // Java notifies every other replay session of the removed sketches and the
                // departing coach via `ServerCommunication.sendToReplaySession` /
                // `sendReplayLeave`, then transfers board control to the first eligible other
                // coach (skipping any coach with sketching prevented). `ReplaySessionManager`
                // has no per-session sender and no tie to `ReplayState.isCoachPreventedFromSketching`
                // wired here yet (Phase ZV), so those network sends and the eligibility filter
                // are not modeled; control is handed to the first other session instead.
                if let Some(&target_session) = other_sessions.first() {
                    if let Some(target_coach) = rsm.coach(target_session) {
                        rsm.transfer_control(session_id, &target_coach);
                    }
                }
            }
            // Java: `getServer().getReplayCache().closeReplay(replayName)` once no sessions
            // remain. `ReplayCache` is not wired into `ffb-server` yet (Phase ZV).

            self.sketch_manager
                .lock()
                .unwrap()
                .remove_session(&session_id.to_string());
        }

        rsm.remove_session(session_id);

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
    use tokio::sync::mpsc;

    fn handler() -> (
        ServerCommandHandlerSocketClosed,
        Arc<Mutex<GameCache>>,
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<ReplaySessionManager>>,
        Arc<Mutex<ServerSketchManager>>,
    ) {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let sm = Arc::new(Mutex::new(SessionManager::new()));
        let rsm = Arc::new(Mutex::new(ReplaySessionManager::new()));
        let sk = Arc::new(Mutex::new(ServerSketchManager::new()));
        let h = ServerCommandHandlerSocketClosed::new(
            Arc::clone(&gc),
            Arc::clone(&sm),
            Arc::clone(&rsm),
            Arc::clone(&sk),
        );
        (h, gc, sm, rsm, sk)
    }

    #[test]
    fn get_id_is_internal_socket_closed() {
        let (h, ..) = handler();
        assert_eq!(h.get_id(), NetCommandId::InternalServerSocketClosed);
    }

    #[test]
    fn close_game_session_removes_session_and_notifies_remaining() {
        let (h, gc, sm, _rsm, _sk) = handler();
        let game_id = gc.lock().unwrap().create_game_state();

        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut guard = sm.lock().unwrap();
            guard.add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
            guard.add_session(2, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        }

        let ok = h.handle_command(1);
        assert!(ok);

        // session 1 removed from bookkeeping
        assert_eq!(sm.lock().unwrap().get_game_id_for_session(1), 0);
        // session 2 (still connected) was notified of the leave
        let msg = rx2.try_recv().expect("expected a leave message");
        let value: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(value["netCommandId"], "serverLeave");
        assert_eq!(value["coach"], "Home");
    }

    #[test]
    fn close_game_session_with_no_remaining_sessions_does_not_panic() {
        let (h, gc, sm, _rsm, _sk) = handler();
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(1, game_id, "Solo".into(), ClientMode::PLAYER, true, vec![], tx);

        let ok = h.handle_command(1);
        assert!(ok);
        assert!(sm.lock().unwrap().get_sessions_for_game_id(game_id).is_empty());
    }

    #[test]
    fn close_replay_session_transfers_control_and_removes_session() {
        let (h, _gc, _sm, rsm, _sk) = handler();
        {
            let mut guard = rsm.lock().unwrap();
            guard.add_session(1, "replay1".into(), "Coach1".into());
            guard.add_session(2, "replay1".into(), "Coach2".into());
        }

        let ok = h.handle_command(1);
        assert!(ok);

        let guard = rsm.lock().unwrap();
        assert!(!guard.has(1));
        assert!(guard.has_control(2));
    }

    #[test]
    fn close_replay_session_last_session_does_not_panic() {
        let (h, _gc, _sm, rsm, _sk) = handler();
        rsm.lock()
            .unwrap()
            .add_session(1, "replay1".into(), "Coach1".into());

        let ok = h.handle_command(1);
        assert!(ok);
        assert!(!rsm.lock().unwrap().has(1));
    }

    #[test]
    fn handle_command_removes_sketch_session_regardless_of_kind() {
        let (h, gc, sm, _rsm, sk) = handler();
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.lock()
            .unwrap()
            .add_session(5, game_id, "Coach".into(), ClientMode::PLAYER, true, vec![], tx);
        sk.lock().unwrap().add_sketch("5", ffb_engine::server_sketch_manager::Sketch::new("s"));

        h.handle_command(5);

        assert_eq!(sk.lock().unwrap().get_sketches("5").len(), 0);
    }
}
