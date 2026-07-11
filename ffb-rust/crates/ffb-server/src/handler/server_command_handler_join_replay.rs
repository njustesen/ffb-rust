/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoinReplay.
use std::sync::{Arc, Mutex};
use ffb_engine::replay_cache::ReplayCache;
use ffb_engine::replay_state::ReplayState;
use ffb_engine::server_sketch_manager::ServerSketchManager;
use ffb_model::enums::NetCommandId;
use ffb_model::model::ClientMode;
use ffb_model::types::FieldCoordinate;
use ffb_protocol::commands::client_command_join_replay::ClientCommandJoinReplay;
use ffb_protocol::commands::server_command_add_sketches::ServerCommandAddSketches;
use ffb_protocol::commands::server_command_join::ServerCommandJoin;
use ffb_protocol::commands::server_command_replay_control::ServerCommandReplayControl;
use ffb_protocol::commands::server_command_replay_status::ServerCommandReplayStatus;
use ffb_protocol::commands::server_command_set_prevent_sketching::ServerCommandSetPreventSketching;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

/// Java: `Constant.REPLAY_NAME_MAX_LENGTH`.
const REPLAY_NAME_MAX_LENGTH: usize = 20;

/// Java: `ServerCommandHandlerJoinReplay extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoinReplay {
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    replay_cache: Arc<Mutex<ReplayCache>>,
    sketch_manager: Arc<Mutex<ServerSketchManager>>,
}

impl ServerCommandHandlerJoinReplay {
    pub fn new(
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        replay_cache: Arc<Mutex<ReplayCache>>,
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
    ) -> Self {
        Self { replay_session_manager, replay_cache, sketch_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.CLIENT_JOIN_REPLAY`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientJoinReplay
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// synchronized (sessionManager) {
    ///     String plainReplayName = clientCommandJoinReplay.getReplayName();
    ///     String sanitizedReplayName = plainReplayName.substring(0, min(REPLAY_NAME_MAX_LENGTH, plainReplayName.length()));
    ///     String replayName = plainReplayName + "_" + clientCommandJoinReplay.getGameId();
    ///     sessionManager.addSession(session, replayName, clientCommandJoinReplay.getCoach());
    ///     String coach = sessionManager.coach(session);
    ///     Session[] sessions = sessionManager.sessionsForReplay(replayName);
    ///     if (ArrayTool.isProvided(sessions)) {
    ///         List<String> coaches = ...;
    ///         sessions.forEach(s -> communication.send(s, new ServerCommandJoin(coach, ClientMode.REPLAY, [], coaches, sanitizedReplayName), true));
    ///     }
    ///     ReplayState replayState = replayCache.replayState(replayName);
    ///     if (replayState == null) { ... new ReplayState, send ServerCommandReplayControl ... }
    ///     else { ... send ReplayStatus/ReplayControl/SetPreventSketching/AddSketches per session ... }
    /// }
    /// return true;
    /// ```
    ///
    /// All bookkeeping and every wire command Java sends are ported for real. One precise,
    /// documented fidelity gap: `ffb_engine::server_sketch_manager::Sketch` (id/coordinates/
    /// label/rgb) and `ffb_model::model::sketch::sketch::Sketch` (the type
    /// `ServerCommandAddSketches` actually carries — positions only) are different structs
    /// from different translation phases; converting one to the other can only carry over
    /// the coordinate list; `id`/`label`/`rgb` have no field to land in on the wire-protocol
    /// side and are dropped. Unifying the two `Sketch` models is a real, separate follow-up
    /// (out of scope here); nothing is fabricated, the position data itself is real.
    pub fn handle_command(&self, cmd: &ClientCommandJoinReplay, session_id: SessionId) -> bool {
        let plain_replay_name = cmd.get_replay_name().unwrap_or_default().to_string();
        let sanitized_replay_name: String = plain_replay_name.chars().take(REPLAY_NAME_MAX_LENGTH).collect();
        let replay_name = format!("{}_{}", plain_replay_name, cmd.get_game_id());

        let mut rsm = self.replay_session_manager.lock().unwrap();
        rsm.add_session(session_id, replay_name.clone(), cmd.get_coach().unwrap_or_default().to_string());
        let coach = rsm.coach(session_id).unwrap_or_default();
        let sessions = rsm.sessions_for_replay(&replay_name).unwrap_or_default();

        if !sessions.is_empty() {
            let coaches: Vec<String> = sessions.iter().filter_map(|&s| rsm.coach(s)).collect();
            let join_cmd = ServerCommandJoin::new(coach.clone(), ClientMode::REPLAY, vec![], coaches, sanitized_replay_name.clone());
            let json = join_cmd.to_json_value().to_string();
            for &s in &sessions {
                rsm.send_to(s, &json);
            }
        }

        let mut cache = self.replay_cache.lock().unwrap();
        let already_cached = cache.replay_state(&replay_name).is_some();

        if !already_cached {
            cache.add(ReplayState::new(&replay_name));
            let control_cmd = ServerCommandReplayControl::new(coach.clone());
            rsm.send_to(session_id, &control_cmd.to_json_value().to_string());
        } else {
            let (command_nr, speed, running, forward, self_prevented) = {
                let state = cache.replay_state(&replay_name).unwrap();
                (
                    state.get_command_nr(),
                    state.get_speed(),
                    state.is_running(),
                    state.is_forward(),
                    state.is_coach_prevented_from_sketching(&coach),
                )
            };

            let status_cmd = ServerCommandReplayStatus::new(command_nr, speed, running, forward, true);
            rsm.send_to(session_id, &status_cmd.to_json_value().to_string());

            let controlling_coach = rsm.controlling_coach(session_id);
            let control_cmd = ServerCommandReplayControl::new(controlling_coach);
            rsm.send_to(session_id, &control_cmd.to_json_value().to_string());

            if self_prevented {
                let prevent_cmd = ServerCommandSetPreventSketching::new(coach.clone(), true);
                rsm.send_to(session_id, &prevent_cmd.to_json_value().to_string());
            }

            let mut sketch_manager = self.sketch_manager.lock().unwrap();
            for other_session in rsm.other_sessions(session_id) {
                let other_coach = rsm.coach(other_session).unwrap_or_default();
                let other_prevented = cache
                    .replay_state(&replay_name)
                    .map(|s| s.is_coach_prevented_from_sketching(&other_coach))
                    .unwrap_or(false);
                if other_prevented {
                    let prevent_cmd = ServerCommandSetPreventSketching::new(other_coach.clone(), true);
                    rsm.send_to(session_id, &prevent_cmd.to_json_value().to_string());
                }

                let sketches = sketch_manager
                    .get_sketches(&other_session.to_string())
                    .iter()
                    .map(|engine_sketch| {
                        let mut model_sketch = ffb_model::model::sketch::sketch::Sketch::new();
                        for &(x, y) in engine_sketch.coordinates() {
                            model_sketch.add_position(FieldCoordinate::new(x, y));
                        }
                        model_sketch
                    })
                    .collect();
                let add_sketches_cmd = ServerCommandAddSketches::new(other_coach, sketches);
                rsm.send_to(session_id, &add_sketches_cmd.to_json_value().to_string());
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Arc<Mutex<ReplaySessionManager>>, Arc<Mutex<ReplayCache>>, Arc<Mutex<ServerSketchManager>>) {
        (
            Arc::new(Mutex::new(ReplaySessionManager::new())),
            Arc::new(Mutex::new(ReplayCache::new())),
            Arc::new(Mutex::new(ServerSketchManager::new())),
        )
    }

    fn handler(
        rsm: Arc<Mutex<ReplaySessionManager>>,
        cache: Arc<Mutex<ReplayCache>>,
        sketches: Arc<Mutex<ServerSketchManager>>,
    ) -> ServerCommandHandlerJoinReplay {
        ServerCommandHandlerJoinReplay::new(rsm, cache, sketches)
    }

    #[test]
    fn construct() {
        let (rsm, cache, sketches) = setup();
        let _ = handler(rsm, cache, sketches);
    }

    #[test]
    fn get_id_is_client_join_replay() {
        let (rsm, cache, sketches) = setup();
        let h = handler(rsm, cache, sketches);
        assert_eq!(h.get_id(), NetCommandId::ClientJoinReplay);
    }

    #[test]
    fn first_session_joining_creates_replay_state_and_gets_control() {
        use tokio::sync::mpsc;
        let (rsm, cache, sketches) = setup();
        let (tx, mut rx) = mpsc::unbounded_channel();
        rsm.lock().unwrap().register_sender(1, tx);
        let h = handler(Arc::clone(&rsm), Arc::clone(&cache), sketches);
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("MyReplay".into()),
            coach: Some("Alice".into()),
            game_id: 42,
        };
        assert!(h.handle_command(&cmd, 1));

        {
            let manager = rsm.lock().unwrap();
            assert!(manager.has(1));
            assert_eq!(manager.coach(1), Some("Alice".to_string()));
            assert_eq!(manager.replay_name_for_session(1), "MyReplay_42");
        }
        assert_eq!(cache.lock().unwrap().replay_count(), 1);

        // First (and only) session in the replay: gets the ServerCommandJoin broadcast plus
        // the new-replay ServerCommandReplayControl.
        let join_msg = rx.try_recv().expect("expected serverJoin broadcast");
        assert!(join_msg.contains("serverJoin"));
        let control_msg = rx.try_recv().expect("expected serverReplayControl");
        assert!(control_msg.contains("serverReplayControl"));
        assert!(control_msg.contains("Alice"));
    }

    #[test]
    fn second_session_joining_existing_replay_gets_status_and_control() {
        use tokio::sync::mpsc;
        let (rsm, cache, sketches) = setup();

        let (tx1, mut rx1) = mpsc::unbounded_channel();
        rsm.lock().unwrap().register_sender(1, tx1);
        let h = handler(Arc::clone(&rsm), Arc::clone(&cache), Arc::clone(&sketches));
        let cmd1 = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("Shared".into()),
            coach: Some("Alice".into()),
            game_id: 7,
        };
        assert!(h.handle_command(&cmd1, 1));
        // Drain session 1's messages from its own join.
        while rx1.try_recv().is_ok() {}

        let (tx2, mut rx2) = mpsc::unbounded_channel();
        rsm.lock().unwrap().register_sender(2, tx2);
        let cmd2 = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("Shared".into()),
            coach: Some("Bob".into()),
            game_id: 7,
        };
        assert!(h.handle_command(&cmd2, 2));

        // Session 2 hits the "existing replay" branch: ReplayStatus, ReplayControl, then
        // (unconditionally, no sketches drawn yet) an empty AddSketches for session 1.
        let join_msg = rx2.try_recv().expect("expected serverJoin broadcast to session 2");
        assert!(join_msg.contains("serverJoin"));
        let status_msg = rx2.try_recv().expect("expected serverReplayStatus");
        assert!(status_msg.contains("serverReplayStatus"));
        let control_msg = rx2.try_recv().expect("expected serverReplayControl naming the controller");
        assert!(control_msg.contains("serverReplayControl"));
        assert!(control_msg.contains("Alice"), "Alice joined first and should still control");
        let add_sketches_msg = rx2.try_recv().expect("expected AddSketches for the other session");
        assert!(add_sketches_msg.contains("serverAddSketches"));
        assert!(add_sketches_msg.contains("Alice"));

        assert_eq!(cache.lock().unwrap().replay_count(), 1, "no second ReplayState is created for the same name");

        // Session 1 also received the broadcasted ServerCommandJoin for session 2 joining.
        let join_for_session1 = rx1.try_recv().expect("expected serverJoin broadcast to session 1 too");
        assert!(join_for_session1.contains("serverJoin"));
    }

    #[test]
    fn prevented_coach_gets_set_prevent_sketching_on_rejoin() {
        use tokio::sync::mpsc;
        let (rsm, cache, sketches) = setup();

        let (tx1, _rx1) = mpsc::unbounded_channel();
        rsm.lock().unwrap().register_sender(1, tx1);
        let h = handler(Arc::clone(&rsm), Arc::clone(&cache), Arc::clone(&sketches));
        let cmd1 = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("Shared".into()),
            coach: Some("Alice".into()),
            game_id: 7,
        };
        assert!(h.handle_command(&cmd1, 1));

        cache.lock().unwrap().replay_state_mut("Shared_7").unwrap().prevent_coach_from_sketching("Alice");

        let (tx1b, mut rx1b) = mpsc::unbounded_channel();
        rsm.lock().unwrap().register_sender(1, tx1b);
        // Alice re-joins the same replay (still session 1) — now prevented from sketching.
        assert!(h.handle_command(&cmd1, 1));

        let mut saw_prevent = false;
        while let Ok(msg) = rx1b.try_recv() {
            if msg.contains("serverSetPreventSketching") && msg.contains("Alice") {
                saw_prevent = true;
            }
        }
        assert!(saw_prevent, "expected a SetPreventSketching(Alice, true) message");
    }
}
