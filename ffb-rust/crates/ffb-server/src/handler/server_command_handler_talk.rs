/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerTalk.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_talk::ClientCommandTalk;
use ffb_protocol::commands::server_command_talk::ServerCommandTalk;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerTalk extends ServerCommandHandler`.
///
/// Java's `handlers: Set<TalkHandler>` (populated via classpath `Scanner`) is
/// omitted here: `crate::handler::talk` is owned by another agent and its
/// `TalkHandler` subclasses are not (yet) exposed for external use, so the
/// `handlers.stream().anyMatch(...)` branch always falls through to the
/// regular chat broadcast, matching Java's behaviour when no handler matches.
pub struct ServerCommandHandlerTalk {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerTalk {
    /// Java: `protected ServerCommandHandlerTalk(FantasyFootballServer server)`
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        session_manager: Arc<Mutex<SessionManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    ) -> Self {
        Self { game_cache, session_manager, replay_session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_TALK`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientTalk
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    pub fn handle_command(&self, session_id: SessionId, talk_command: &ClientCommandTalk) -> bool {
        if let Some(talk) = talk_command.get_talk() {
            let is_replay = self.replay_session_manager.lock().unwrap().has(session_id);
            if is_replay {
                self.handle_replay_talk(session_id, talk);
            } else {
                self.handle_game_talk(session_id, talk);
            }
        }
        true
    }

    /// Java: `private void handleReplayTalk(Session, ReplaySessionManager, String)`.
    fn handle_replay_talk(&self, session_id: SessionId, talk: &str) {
        let rsm = self.replay_session_manager.lock().unwrap();
        let replay_name = rsm.replay_name_for_session(session_id);
        let coach = match rsm.coach(session_id) {
            Some(c) => c,
            None => return,
        };
        // Java checks `replayState == null` via ReplayCache; the Rust MVP has
        // no wired ReplayCache lookup here, so we fall back to checking that
        // the session actually maps to a known replay name.
        if replay_name.is_empty() {
            return;
        }

        // Java: `ServerCommandTalk.Mode.REGULAR` — replay talk is always REGULAR mode.
        let msg = ServerCommandTalk::new(coach, vec![talk.to_string()], "REGULAR");
        let json = talk_to_json(&msg);
        let sm = self.session_manager.lock().unwrap();
        sm.send_to(session_id, &json);
        for other in rsm.other_sessions(session_id) {
            sm.send_to(other, &json);
        }
    }

    /// Java: `private void handleGameTalk(Session, ClientCommandTalk, String)`.
    fn handle_game_talk(&self, session_id: SessionId, talk: &str) {
        let sm = self.session_manager.lock().unwrap();
        let game_id = sm.get_game_id_for_session(session_id);
        let coach = match sm.get_coach_for_session(session_id) {
            Some(c) => c.to_string(),
            None => return,
        };

        {
            let gc = self.game_cache.lock().unwrap();
            if gc.get_game_state_by_id(game_id).is_none() {
                return;
            }
        }

        let is_home = sm.get_session_of_home_coach(game_id) == Some(session_id);
        let is_away = sm.get_session_of_away_coach(game_id) == Some(session_id);
        let is_admin = sm.is_session_admin(session_id);
        let is_dev = sm.is_session_dev(session_id);

        // Java: `if (getServer().getMode() == ServerMode.FUMBBL) { requestProcessor.add(new
        // FumbblRequestUploadTalk(...)) }` — FUMBBL request-upload requires an HTTP request
        // queue that isn't wired in the Rust MVP (standalone-only), so it is intentionally
        // not reached here.

        // Java: `handlers.stream().anyMatch(handler -> handler.handle(...))` — see struct doc.

        let msg = if is_home || is_away {
            // Java: `communication.sendPlayerTalk(gameState, coach, talk)`.
            ServerCommandTalk::new(coach, vec![talk.to_string()], "REGULAR")
        } else {
            // Java: `ServerCommandTalk.Mode.STAFF`/`DEV` indicator is `"!"`; STAFF takes
            // precedence over DEV per the Java comment.
            let mode = if is_admin && talk.starts_with('!') {
                "STAFF"
            } else if is_dev && talk.starts_with('!') {
                "DEV"
            } else {
                "REGULAR"
            };
            ServerCommandTalk::new(coach, vec![talk.to_string()], mode)
        };

        let json = talk_to_json(&msg);
        sm.send_all(game_id, &json);
    }
}

/// Manual JSON encoding for `ServerCommandTalk` — the `ffb_protocol::commands`
/// structs (as opposed to the `server_commands::ServerCommand` enum) do not
/// derive `Serialize`.
fn talk_to_json(msg: &ServerCommandTalk) -> String {
    serde_json::json!({
        "netCommandId": "serverTalk",
        "coach": msg.get_coach(),
        "talks": msg.get_talks(),
        "mode": msg.get_mode(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn setup() -> (
        Arc<Mutex<GameCache>>,
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<ReplaySessionManager>>,
    ) {
        (
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(ReplaySessionManager::new())),
        )
    }

    #[test]
    fn construct() {
        let (gc, sm, rsm) = setup();
        let _ = ServerCommandHandlerTalk::new(gc, sm, rsm);
    }

    #[test]
    fn get_id_is_client_talk() {
        let (gc, sm, rsm) = setup();
        let handler = ServerCommandHandlerTalk::new(gc, sm, rsm);
        assert_eq!(handler.get_id(), NetCommandId::ClientTalk);
    }

    #[test]
    fn no_talk_text_is_a_noop() {
        let (gc, sm, rsm) = setup();
        let handler = ServerCommandHandlerTalk::new(gc, sm, rsm);
        let result = handler.handle_command(1, &ClientCommandTalk::new());
        assert!(result);
    }

    #[test]
    fn game_talk_from_home_coach_broadcasts_to_all_sessions() {
        let (gc, sm, rsm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut s = sm.lock().unwrap();
            s.add_session(1, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
            s.add_session(2, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        }

        let handler = ServerCommandHandlerTalk::new(gc, sm, rsm);
        let result = handler.handle_command(1, &ClientCommandTalk::with_talk("hello"));
        assert!(result);

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();
        assert!(msg1.contains("Home"));
        assert!(msg1.contains("hello"));
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn game_talk_from_unknown_session_is_ignored() {
        let (gc, sm, rsm) = setup();
        let handler = ServerCommandHandlerTalk::new(gc, sm, rsm);
        // Session 99 was never registered — get_coach_for_session returns None.
        let result = handler.handle_command(99, &ClientCommandTalk::with_talk("hello"));
        assert!(result);
    }

    #[test]
    fn replay_talk_broadcasts_to_other_replay_sessions() {
        let (gc, sm, rsm) = setup();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        {
            let mut s = sm.lock().unwrap();
            // Register senders under an arbitrary game id — replay chat delivery
            // reuses SessionManager's per-session send channel.
            s.add_session(1, 0, "Viewer1".into(), ClientMode::SPECTATOR, false, vec![], tx1);
            s.add_session(2, 0, "Viewer2".into(), ClientMode::SPECTATOR, false, vec![], tx2);
            let mut r = rsm.lock().unwrap();
            r.add_session(1, "replay1".into(), "Viewer1".into());
            r.add_session(2, "replay1".into(), "Viewer2".into());
        }

        let handler = ServerCommandHandlerTalk::new(gc, sm, rsm);
        let result = handler.handle_command(1, &ClientCommandTalk::with_talk("hi replay"));
        assert!(result);

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();
        assert!(msg1.contains("Viewer1"));
        assert!(msg1.contains("hi replay"));
        assert_eq!(msg1, msg2);
    }
}
