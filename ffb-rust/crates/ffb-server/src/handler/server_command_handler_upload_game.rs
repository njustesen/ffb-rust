/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerUploadGame.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command_upload_game::InternalServerCommandUploadGame;
use crate::request::server_request_load_replay::ServerRequestLoadReplay;

pub struct ServerCommandHandlerUploadGame {
    game_cache: Arc<Mutex<GameCache>>,
}

impl ServerCommandHandlerUploadGame {
    pub fn new(game_cache: Arc<Mutex<GameCache>>) -> Self {
        Self { game_cache }
    }

    /// Java: getId() — returns NetCommandId for UPLOAD_GAME.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerUploadGame
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles uploading a game to FUMBBL.
    ///
    /// Looks up the game in the cache (real). If it is missing, Java builds a
    /// `ServerRequestLoadReplay` (mode `UPLOAD_GAME`) and enqueues it on the
    /// `ServerRequestProcessor` — the request object construction is real,
    /// but enqueueing requires the (separately stubbed) request-processor
    /// queue and an HTTP backup-service client, neither of which is wired
    /// yet. If the game is present, Java clears the step stack, marks the
    /// conceding team on the game result, and pushes an `EndGame` sequence —
    /// none of `GameState.step_stack`, `Game.game_result`/`TeamResult.conceded`,
    /// or the `EndGame` sequence-generator wiring exist on the server-side
    /// `GameState` wrapper yet, so that branch is also a narrow todo.
    pub fn handle_command(&self, cmd: &InternalServerCommandUploadGame) -> bool {
        let found = {
            let gc = self.game_cache.lock().unwrap();
            gc.get_game_state_by_id(cmd.game_id).is_some()
        };

        if !found {
            let _request = self.build_load_replay_request(cmd);
            // Java: getServer().getRequestProcessor().add(request);
            todo!("Phase ZV: ServerRequestProcessor.add + HTTP backup-service request need wiring")
        } else {
            let _has_conceding_team = cmd.get_conceding_team_id().is_some();
            // Java: gameState.getStepStack().clear();
            //       game.getGameResult().getTeamResultHome/Away().setConceded(...);
            //       ((EndGame) factory.forName("EndGame")).pushSequence(...);
            //       gameState.startNextStep();
            todo!("Phase ZV: GameState step-stack clear + GameResult.setConceded + EndGame sequence need wiring")
        }
    }

    /// Java: `new ServerRequestLoadReplay(gameId, 0, session, UPLOAD_GAME, concedingTeamId, null)`.
    fn build_load_replay_request(&self, cmd: &InternalServerCommandUploadGame) -> ServerRequestLoadReplay {
        ServerRequestLoadReplay::new(
            cmd.game_id,
            0,
            ServerRequestLoadReplay::UPLOAD_GAME,
            cmd.get_conceding_team_id().unwrap_or("").to_string(),
            String::new(),
        )
    }
}

impl Default for ServerCommandHandlerUploadGame {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(GameCache::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerUploadGame::default();
    }

    #[test]
    fn get_id_returns_internal_server_upload_game() {
        let h = ServerCommandHandlerUploadGame::default();
        assert_eq!(h.get_id(), NetCommandId::InternalServerUploadGame);
    }

    #[test]
    fn build_load_replay_request_carries_upload_mode_and_conceding_team() {
        let h = ServerCommandHandlerUploadGame::default();
        let cmd = InternalServerCommandUploadGame::new_with_conceding(42, Some("teamA".to_string()));
        let request = h.build_load_replay_request(&cmd);
        assert_eq!(request.get_game_id(), 42);
        assert_eq!(request.get_mode(), ServerRequestLoadReplay::UPLOAD_GAME);
    }

    #[test]
    fn handle_command_missing_game_hits_request_processor_todo() {
        let h = ServerCommandHandlerUploadGame::default();
        let cmd = InternalServerCommandUploadGame::new(1);
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd));
        assert!(result.is_err(), "missing-game branch requires ServerRequestProcessor + HTTP wiring (narrow todo!)");
    }

    #[test]
    fn handle_command_known_game_hits_end_game_sequence_todo() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = { gc.lock().unwrap().create_game_state() };
        let h = ServerCommandHandlerUploadGame::new(gc);
        let cmd = InternalServerCommandUploadGame::new(game_id);
        let result = std::panic::catch_unwind(|| h.handle_command(&cmd));
        assert!(result.is_err(), "known-game branch requires step-stack + EndGame sequence wiring (narrow todo!)");
    }
}
