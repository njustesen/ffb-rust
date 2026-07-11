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
    /// yet, so that branch remains a narrow todo (Phase ZZ or later — unrelated
    /// to the step-stack/EndGame wiring closed here in Phase ZY.1).
    pub fn handle_command(&self, cmd: &InternalServerCommandUploadGame) -> bool {
        let mut gc = self.game_cache.lock().unwrap();
        let game_state = gc.get_game_state_by_id_mut(cmd.game_id);

        match game_state {
            None => {
                let _request = self.build_load_replay_request(cmd);
                // Java: getServer().getRequestProcessor().add(request);
                todo!("Phase ZZ: ServerRequestProcessor.add + HTTP backup-service request need wiring")
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
        let h = ServerCommandHandlerUploadGame::new(gc.clone());
        let cmd = InternalServerCommandUploadGame::new(game_id);

        assert!(h.handle_command(&cmd));

        let guard = gc.lock().unwrap();
        let gs = guard.get_game_state_by_id(game_id).unwrap();
        assert!(gs.is_finished());
    }

    #[test]
    fn handle_command_known_game_marks_conceding_team() {
        let gc = Arc::new(Mutex::new(GameCache::new()));
        let game_id = started_game_id(&gc);
        let h = ServerCommandHandlerUploadGame::new(gc.clone());
        let cmd = InternalServerCommandUploadGame::new_with_conceding(game_id, Some("home".to_string()));

        assert!(h.handle_command(&cmd));

        let mut guard = gc.lock().unwrap();
        let gs = guard.get_game_state_by_id_mut(game_id).unwrap();
        let game = gs.get_game().unwrap();
        assert!(game.game_result.home.conceded);
        assert!(!game.game_result.away.conceded);
    }
}
