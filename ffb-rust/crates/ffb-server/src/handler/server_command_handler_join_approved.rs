/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerJoinApproved.
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ffb_model::enums::NetCommandId;
use ffb_model::model::ClientMode;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_join_approved::InternalServerCommandJoinApproved;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommandHandlerJoinApproved.TEST_PREFIX`.
const TEST_PREFIX: &str = "test:";

/// Java: `ServerCommandHandlerJoinApproved extends ServerCommandHandler`.
pub struct ServerCommandHandlerJoinApproved {
    game_cache: Arc<Mutex<GameCache>>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ServerCommandHandlerJoinApproved {
    pub fn new(game_cache: Arc<Mutex<GameCache>>, session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { game_cache, session_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.INTERNAL_SERVER_JOIN_APPROVED`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerJoinApproved
    }

    /// Java: `loadGameStateById(InternalServerCommandJoinApproved)`.
    ///
    /// Java falls back to `GameCache.queryFromDb(gameId)` when the game isn't
    /// cached in memory; the Rust `GameCache` has no persistence layer, so a
    /// cache miss here simply reports "not found" (there is nothing further
    /// to query).
    fn load_game_state_by_id(&self, game_id: i64) -> bool {
        self.game_cache.lock().unwrap().get_game_state_by_id(game_id).is_some()
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// `sender` supplies the outgoing channel for `SessionManager.add_session`
    /// (Java re-uses the already-connected Jetty `Session`; the Rust
    /// `SessionManager` needs the sender explicitly since there's no `Session`
    /// object to query).
    pub fn handle_command(
        &self,
        join_approved_command: &InternalServerCommandJoinApproved,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
    ) -> bool {
        let mut game_id = 0i64;
        let mut game_found = false;

        if join_approved_command.get_game_id() > 0 {
            game_id = join_approved_command.get_game_id();
            game_found = self.load_game_state_by_id(game_id);
        } else if !join_approved_command.get_game_name().is_empty() {
            let game_name = join_approved_command.get_game_name();
            let existing_id = {
                let gc = self.game_cache.lock().unwrap();
                gc.get_game_state_by_name(game_name).map(|gs| gs.get_id())
            };
            match existing_id {
                Some(id) => {
                    game_id = id;
                    game_found = true;
                }
                None => {
                    // Java: `!getServer().isBlockingNewGames()` gate — no equivalent
                    // server-wide flag exists yet, so new games are always created,
                    // matching the common (non-blocking) case.
                    let _testing = game_name.starts_with(TEST_PREFIX);
                    let mut gc = self.game_cache.lock().unwrap();
                    let id = gc.create_game_state();
                    gc.map_game_name_to_id(game_name.to_string(), id);
                    game_id = id;
                    game_found = true;
                }
            }
        }

        if game_found {
            let client_mode = parse_client_mode(join_approved_command.get_client_mode());
            if client_mode == Some(ClientMode::PLAYER) {
                // Java: `joinWithoutTeam` / `joinWithTeam` / `sendTeamList` — depend on
                // `Game.getTeamHome()/getTeamAway()` plus `GameCache.getTeamSkeleton`/
                // `getTeamById`/`addTeamToGame`, all of which now exist for real (Phase
                // ZY.2/ZY.3's XML roster deserializer + `GameCache::get_team_by_id`). The
                // remaining, genuinely different blocker is
                // `UtilServerStartGame::join_game_as_player_and_check_if_ready_to_start`
                // (Phase ZX.3) — it's `async` and needs a `DbConnectionManager`, neither of
                // which this handler's signature threads through. Wiring that in means
                // making `handle_command` async and adding a DB parameter to this struct's
                // constructor — a separate, larger infra change from roster resolution,
                // narrowly scoped out of this phase rather than attempted here.
                todo!(
                    "Phase ZY.4+: needs handle_command made async + DbConnectionManager threaded \
                     through, to call join_game_as_player_and_check_if_ready_to_start / \
                     addDefaultGameOptions / getTeamById(x2)+addTeamToGame(x2) / start_game (game_id = {})",
                    game_id
                );
            } else {
                // ClientMode.SPECTATOR: close any other session for this coach,
                // then register this one as a spectator.
                let coach = join_approved_command.get_coach().to_string();
                {
                    let mut sm = self.session_manager.lock().unwrap();
                    if let Some(other) = sm.find_other_session_for_coach(game_id, &coach, session_id) {
                        sm.remove_session(other);
                    }
                    sm.add_session(
                        session_id,
                        game_id,
                        coach,
                        ClientMode::SPECTATOR,
                        false,
                        join_approved_command.get_account_properties().to_vec(),
                        sender,
                    );
                }
                // Java: `UtilServerStartGame.sendServerJoin(...)` and, if the game has already
                // started, `UtilServerTimer.syncTime` + `communication.sendGameState`.
                // `send_server_join` now exists for real (Phase ZX.3,
                // `util::server_start_game::send_server_join`) but is `async` and needs a
                // `DbConnectionManager` — the same handler-signature gap as the PLAYER branch
                // above, not a roster-resolution gap.
                todo!(
                    "Phase ZY.4+: needs handle_command made async + DbConnectionManager threaded \
                     through, to call send_server_join / UtilServerTimer.syncTime / sendGameState (game_id = {})",
                    game_id
                );
            }
        } else if parse_client_mode(join_approved_command.get_client_mode()) == Some(ClientMode::REPLAY) {
            // Java: `UtilServerStartGame.sendUserSettings(...)`. `send_user_settings` now
            // exists for real (Phase ZX.3) but is `async` and needs a `DbConnectionManager` —
            // same handler-signature gap as above.
            todo!(
                "Phase ZY.4+: needs handle_command made async + DbConnectionManager threaded \
                 through, to call send_user_settings"
            );
        }

        true
    }
}

/// Java's `ClientMode` enum constant name (`"PLAYER"`, `"SPECTATOR"`,
/// `"REPLAY"`), stored as a plain `String` on
/// `InternalServerCommandJoinApproved` pending full enum wiring there.
fn parse_client_mode(raw: &str) -> Option<ClientMode> {
    match raw.to_ascii_uppercase().as_str() {
        "PLAYER" => Some(ClientMode::PLAYER),
        "SPECTATOR" => Some(ClientMode::SPECTATOR),
        "REPLAY" => Some(ClientMode::REPLAY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Arc<Mutex<GameCache>>, Arc<Mutex<SessionManager>>) {
        (
            Arc::new(Mutex::new(GameCache::new())),
            Arc::new(Mutex::new(SessionManager::new())),
        )
    }

    #[test]
    fn construct() {
        let (gc, sm) = setup();
        let _ = ServerCommandHandlerJoinApproved::new(gc, sm);
    }

    #[test]
    fn get_id_is_internal_server_join_approved() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerJoinApproved::new(gc, sm);
        assert_eq!(handler.get_id(), NetCommandId::InternalServerJoinApproved);
    }

    #[test]
    fn spectator_join_creates_game_by_name_and_registers_session_before_broadcast_stub() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerJoinApproved::new(Arc::clone(&gc), Arc::clone(&sm));
        let cmd = InternalServerCommandJoinApproved::new(
            0,
            "NewGame".to_string(),
            "Watcher".to_string(),
            String::new(),
            "SPECTATOR".to_string(),
            vec![],
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.handle_command(&cmd, 7, tx)
        }));
        assert!(result.is_err());

        // The game should have been created and mapped by name...
        assert!(gc.lock().unwrap().get_game_state_by_name("NewGame").is_some());
        // ...and the spectator session registered before the broadcast stub panicked.
        let sm = sm.lock().unwrap();
        assert_eq!(sm.get_coach_for_session(7), Some("Watcher"));
        assert_eq!(sm.get_mode_for_session(7), Some(ClientMode::SPECTATOR));
    }

    #[test]
    fn spectator_join_closes_existing_session_for_same_coach() {
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        gc.lock().unwrap().map_game_name_to_id("ExistingGame".to_string(), game_id);
        {
            let (tx, _rx) = mpsc::unbounded_channel();
            sm.lock().unwrap().add_session(1, game_id, "Watcher".into(), ClientMode::SPECTATOR, false, vec![], tx);
        }
        let handler = ServerCommandHandlerJoinApproved::new(Arc::clone(&gc), Arc::clone(&sm));
        let cmd = InternalServerCommandJoinApproved::new(
            0,
            "ExistingGame".to_string(),
            "Watcher".to_string(),
            String::new(),
            "SPECTATOR".to_string(),
            vec![],
        );
        let (tx2, _rx2) = mpsc::unbounded_channel();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.handle_command(&cmd, 2, tx2)
        }));
        assert!(result.is_err());
        let sm = sm.lock().unwrap();
        assert_eq!(sm.get_game_id_for_session(1), 0, "old session should have been closed");
        assert_eq!(sm.get_coach_for_session(2), Some("Watcher"));
    }

    #[test]
    fn player_join_by_existing_game_id_hits_team_infra_stub() {
        let (gc, sm) = setup();
        let game_id = gc.lock().unwrap().create_game_state();
        let handler = ServerCommandHandlerJoinApproved::new(gc, sm);
        let cmd = InternalServerCommandJoinApproved::new(
            game_id,
            String::new(),
            "Coach".to_string(),
            "team1".to_string(),
            "PLAYER".to_string(),
            vec![],
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.handle_command(&cmd, 3, tx)
        }));
        assert!(result.is_err());
    }

    #[test]
    fn replay_mode_with_no_matching_game_hits_user_settings_stub() {
        let (gc, sm) = setup();
        let handler = ServerCommandHandlerJoinApproved::new(gc, sm);
        let cmd = InternalServerCommandJoinApproved::new(
            0,
            String::new(),
            "Coach".to_string(),
            String::new(),
            "REPLAY".to_string(),
            vec![],
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.handle_command(&cmd, 4, tx)
        }));
        assert!(result.is_err());
    }

    #[test]
    fn parse_client_mode_recognizes_all_variants() {
        assert_eq!(parse_client_mode("PLAYER"), Some(ClientMode::PLAYER));
        assert_eq!(parse_client_mode("spectator"), Some(ClientMode::SPECTATOR));
        assert_eq!(parse_client_mode("Replay"), Some(ClientMode::REPLAY));
        assert_eq!(parse_client_mode("bogus"), None);
    }
}
