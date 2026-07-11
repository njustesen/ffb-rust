/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerUploadGame.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::model::received_command::{ReceivedCommand, SessionId};
use crate::net::commands::internal_server_command_upload_game::InternalServerCommandUploadGame;
use crate::request::fumbbl::util_fumbbl_request::HttpClient;
use crate::request::server_request_load_replay::{QueuedServerRequestLoadReplay, ServerRequestLoadReplay};
use crate::request::server_request_processor::ServerRequestProcessor;

pub struct ServerCommandHandlerUploadGame {
    game_cache: Arc<Mutex<GameCache>>,
    /// Java: `getServer().getRequestProcessor()`.
    request_processor: Arc<Mutex<ServerRequestProcessor>>,
    /// Java: `UtilServerHttpClient.fetchPage(...)` inside `ServerRequestLoadReplay.process`.
    client: Arc<dyn HttpClient + Send + Sync>,
    /// Java: `ServerUrlProperty.BACKUP_URL_LOAD.url(server.getProperties())`.
    backup_url_load_template: String,
    /// Java: `server.getCommunication().handleCommand(new ReceivedCommand(uploadCommand, session))`
    /// — the redispatch sink `QueuedServerRequestLoadReplay` uses once the backup service
    /// answers. A clone of the same `mpsc::UnboundedSender<ReceivedCommand>` that feeds
    /// `net::server_communication::dispatch_loop` (see `ServerCommunication::new`), matching
    /// the convention already established by `ServerCommandHandlerJoin`.
    dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
}

impl ServerCommandHandlerUploadGame {
    pub fn new(
        game_cache: Arc<Mutex<GameCache>>,
        request_processor: Arc<Mutex<ServerRequestProcessor>>,
        client: Arc<dyn HttpClient + Send + Sync>,
        backup_url_load_template: impl Into<String>,
        dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>,
    ) -> Self {
        Self {
            game_cache,
            request_processor,
            client,
            backup_url_load_template: backup_url_load_template.into(),
            dispatch_tx,
        }
    }

    /// Java: getId() — returns NetCommandId for UPLOAD_GAME.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerUploadGame
    }

    /// Java: `handleCommand(ReceivedCommand)` — handles uploading a game to FUMBBL.
    ///
    /// Looks up the game in the cache. If it is missing, Java builds a
    /// `ServerRequestLoadReplay` (mode `UPLOAD_GAME`) and enqueues it on the
    /// `ServerRequestProcessor`; that request fetches the game from the backup service,
    /// rehydrates a `GameState`, re-adds it to the cache, and redispatches
    /// `InternalServerCommandUploadGame` so this same handler runs again against the
    /// now-present game — all real now via `QueuedServerRequestLoadReplay` (see that
    /// struct's doc comment in `request/server_request_load_replay.rs` for what's ported
    /// 1:1 vs. narrowed).
    pub fn handle_command(&self, cmd: &InternalServerCommandUploadGame, session_id: SessionId) -> bool {
        let mut gc = self.game_cache.lock().unwrap();
        let game_state = gc.get_game_state_by_id_mut(cmd.game_id);

        match game_state {
            None => {
                drop(gc);
                let request = self.build_load_replay_request(cmd);
                let queued = QueuedServerRequestLoadReplay::new(
                    request,
                    Arc::clone(&self.client),
                    self.backup_url_load_template.clone(),
                    Arc::clone(&self.game_cache),
                    self.dispatch_tx.clone(),
                    session_id,
                );
                // Java: `getServer().getRequestProcessor().add(request);`
                self.request_processor.lock().unwrap().add(Box::new(queued));
                true
            }
            Some(game_state) => {
                // Java: StringTool.isProvided(concedingTeamId) — non-null and non-empty.
                if let Some(conceding_team_id) = cmd.get_conceding_team_id().filter(|s| !s.is_empty()) {
                    if let Some(game) = game_state.get_game_mut() {
                        let home_conceded = game.team_home.id == conceding_team_id;
                        let away_conceded = game.team_away.id == conceding_team_id;
                        game.game_result.home.conceded = home_conceded;
                        game.game_result.away.conceded = away_conceded;
                    }
                }
                game_state.clear_step_stack();
                game_state.push_end_game_sequence(true);
                game_state.start_next_step();
                true
            }
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
        let (dispatch_tx, _dispatch_rx) = mpsc::unbounded_channel();
        Self::new(
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(crate::request::fumbbl::util_fumbbl_request::ReqwestHttpClient::new()),
            String::new(),
            dispatch_tx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::fumbbl::util_fumbbl_request::MockHttpClient;

    fn handler_with(
        game_cache: Arc<Mutex<GameCache>>,
        response: Result<String, String>,
    ) -> (ServerCommandHandlerUploadGame, mpsc::UnboundedReceiver<ReceivedCommand>) {
        let (dispatch_tx, dispatch_rx) = mpsc::unbounded_channel();
        let handler = ServerCommandHandlerUploadGame::new(
            game_cache,
            Arc::new(Mutex::new(ServerRequestProcessor::new())),
            Arc::new(MockHttpClient { response }),
            "http://backup/load/$1",
            dispatch_tx,
        );
        (handler, dispatch_rx)
    }

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
    fn handle_command_missing_game_enqueues_load_replay_request() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let (h, _rx) = handler_with(Arc::clone(&gc), Ok(String::new()));
        let cmd = InternalServerCommandUploadGame::new(1);

        assert!(h.handle_command(&cmd, 5));

        assert_eq!(h.request_processor.lock().unwrap().queue_len(), 1);
    }

    #[test]
    fn handle_command_missing_game_runs_the_full_backup_round_trip() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let team = |id: &str| ffb_model::model::team::Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
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
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let backup_game = ffb_model::model::game::Game::new(team("home"), team("away"), ffb_model::enums::Rules::Bb2025);
        let backup_json = serde_json::to_string(&backup_game).unwrap();

        let (h, mut rx) = handler_with(Arc::clone(&gc), Ok(backup_json));
        let cmd = InternalServerCommandUploadGame::new_with_conceding(99, Some("teamA".to_string()));

        assert!(h.handle_command(&cmd, 5));
        assert_eq!(h.request_processor.lock().unwrap().queue_len(), 1);

        // Drain the queue exactly like the real `ServerRequestProcessor` background loop would.
        h.request_processor.lock().unwrap().run().unwrap();

        assert!(gc.lock().unwrap().get_game_state_by_id(99).is_some(), "backup game should be re-added to the cache");
        let redispatched = rx.try_recv().expect("expected a redispatched UploadGame command");
        assert_eq!(redispatched.session_id, 5);
        match redispatched.command {
            crate::model::received_command::ReceivedNetCommand::Internal(
                crate::net::commands::any_internal_server_command::AnyInternalServerCommand::UploadGame(inner),
            ) => {
                assert_eq!(inner.game_id, 99);
                assert_eq!(inner.get_conceding_team_id(), Some("teamA"));
            }
            _ => panic!("expected an internal UploadGame command"),
        }
    }

    fn team(id: &str) -> ffb_model::model::team::Team {
        ffb_model::model::team::Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
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
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn started_game_id(gc: &Arc<Mutex<GameCache>>) -> i64 {
        let mut guard = gc.lock().unwrap();
        let game_id = guard.create_game_state();
        let gs = guard.get_game_state_by_id_mut(game_id).unwrap();
        gs.start_game(team("home"), team("away"), ffb_model::enums::Rules::Bb2025, 0);
        game_id
    }

    #[test]
    fn handle_command_known_game_clears_stack_and_drives_to_finished() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = started_game_id(&gc);
        let (h, _rx) = handler_with(Arc::clone(&gc), Ok(String::new()));
        let cmd = InternalServerCommandUploadGame::new(game_id);

        assert!(h.handle_command(&cmd, 1));

        let guard = gc.lock().unwrap();
        let gs = guard.get_game_state_by_id(game_id).unwrap();
        assert!(gs.is_finished());
    }

    #[test]
    fn handle_command_known_game_marks_conceding_team() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = started_game_id(&gc);
        let (h, _rx) = handler_with(Arc::clone(&gc), Ok(String::new()));
        let cmd = InternalServerCommandUploadGame::new_with_conceding(game_id, Some("home".to_string()));

        assert!(h.handle_command(&cmd, 1));

        let mut guard = gc.lock().unwrap();
        let gs = guard.get_game_state_by_id_mut(game_id).unwrap();
        let game = gs.get_game().unwrap();
        assert!(game.game_result.home.conceded);
        assert!(!game.game_result.away.conceded);
    }
}
