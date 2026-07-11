/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoinReplay.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::client_command_join_replay::ClientCommandJoinReplay;
use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

/// Java: `Constant.REPLAY_NAME_MAX_LENGTH`.
const REPLAY_NAME_MAX_LENGTH: usize = 20;

/// Java: `ServerCommandHandlerJoinReplay extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoinReplay {
    replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
}

impl ServerCommandHandlerJoinReplay {
    pub fn new(replay_session_manager: Arc<Mutex<ReplaySessionManager>>) -> Self {
        Self { replay_session_manager }
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
    /// The session bookkeeping (`ReplaySessionManager.add_session`, coach
    /// lookup, sibling-session/coaches collection) is ported for real. The
    /// broadcast + `ReplayCache`/`ReplayState`/`ServerSketchManager`-dependent
    /// tail of the method has no Rust equivalent yet (no `ReplayCache`,
    /// `ReplayState`, sketch manager, or the `ServerCommandJoin` /
    /// `ServerCommandReplayControl` / `ServerCommandReplayStatus` /
    /// `ServerCommandSetPreventSketching` / `ServerCommandAddSketches` wire
    /// commands), so it remains a narrow `todo!()`.
    pub fn handle_command(&self, client_command_join_replay: &ClientCommandJoinReplay, session_id: SessionId) -> bool {
        let plain_replay_name = client_command_join_replay.get_replay_name().unwrap_or_default().to_string();
        let sanitized_replay_name: String = plain_replay_name.chars().take(REPLAY_NAME_MAX_LENGTH).collect();
        let replay_name = format!("{}_{}", plain_replay_name, client_command_join_replay.get_game_id());

        let (coach, coaches) = {
            let mut rsm = self.replay_session_manager.lock().unwrap();
            rsm.add_session(
                session_id,
                replay_name.clone(),
                client_command_join_replay.get_coach().unwrap_or_default().to_string(),
            );
            let coach = rsm.coach(session_id).unwrap_or_default();
            let sessions = rsm.sessions_for_replay(&replay_name).unwrap_or_default();
            let coaches: Vec<String> = sessions.iter().filter_map(|&s| rsm.coach(s)).collect();
            (coach, coaches)
        };

        // Java: broadcast `ServerCommandJoin` to `sessions`, then either start a
        // new `ReplayState` (send `ServerCommandReplayControl`) or resume an
        // existing one (send `ReplayStatus`/`ReplayControl`/
        // `SetPreventSketching`/`AddSketches` per session).
        todo!(
            "Phase ZV: needs ReplayCache/ReplayState/ServerSketchManager + replay wire commands \
             (replay_name = {}, sanitized = {}, coach = {}, coaches = {:?})",
            replay_name, sanitized_replay_name, coach, coaches
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Arc<Mutex<ReplaySessionManager>> {
        Arc::new(Mutex::new(ReplaySessionManager::new()))
    }

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerJoinReplay::new(setup());
    }

    #[test]
    fn get_id_is_client_join_replay() {
        let handler = ServerCommandHandlerJoinReplay::new(setup());
        assert_eq!(handler.get_id(), NetCommandId::ClientJoinReplay);
    }

    #[test]
    fn joining_registers_session_before_hitting_replay_state_stub() {
        let rsm = setup();
        let handler = ServerCommandHandlerJoinReplay::new(Arc::clone(&rsm));
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("MyReplay".into()),
            coach: Some("Alice".into()),
            game_id: 42,
        };
        let result = std::panic::catch_unwind(|| handler.handle_command(&cmd, 1));
        assert!(result.is_err());
        let manager = rsm.lock().unwrap();
        assert!(manager.has(1));
        assert_eq!(manager.coach(1), Some("Alice".to_string()));
        assert_eq!(manager.replay_name_for_session(1), "MyReplay_42");
    }

    #[test]
    fn second_session_sees_first_coach_in_replay_before_stub() {
        let rsm = setup();
        {
            let mut manager = rsm.lock().unwrap();
            manager.add_session(1, "Existing_1".into(), "Alice".into());
        }
        let handler = ServerCommandHandlerJoinReplay::new(Arc::clone(&rsm));
        let cmd = ClientCommandJoinReplay {
            entropy: None,
            replay_name: Some("Existing".into()),
            coach: Some("Bob".into()),
            game_id: 1,
        };
        let result = std::panic::catch_unwind(|| handler.handle_command(&cmd, 2));
        assert!(result.is_err());
        let manager = rsm.lock().unwrap();
        assert!(manager.has(2));
        assert_eq!(manager.coach(2), Some("Bob".to_string()));
        // Both coaches are now registered for the shared replay.
        let sessions = manager.sessions_for_replay("Existing_1").unwrap();
        assert_eq!(sessions.len(), 2);
    }
}
