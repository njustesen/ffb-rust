/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerDeleteGame.
use std::sync::{Arc, Mutex};
use ffb_model::enums::NetCommandId;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::delete::db_games_info_delete::DbGamesInfoDelete;
use crate::db::delete::db_games_serialized_delete::DbGamesSerializedDelete;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_delete_game::InternalServerCommandDeleteGame;

/// Java: `ServerCommandHandlerDeleteGame extends ServerCommandHandler`.
///
/// Java's `handleCommand` calls `getServer().getGameCache().queueDbDelete(...)` — a method on
/// `GameCache`, but one that (per its Java body) only issues DB statements and never touches
/// `GameCache`'s own in-memory maps. So unlike other handlers here, no `GameCache` reference is
/// needed to reproduce its logic.
pub struct ServerCommandHandlerDeleteGame {
    db_connection_manager: Arc<Mutex<DbConnectionManager>>,
}

impl ServerCommandHandlerDeleteGame {
    pub fn new(db_connection_manager: Arc<Mutex<DbConnectionManager>>) -> Self {
        Self { db_connection_manager }
    }

    /// Java: `getId()` — returns `NetCommandId.INTERNAL_SERVER_DELETE_GAME`.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerDeleteGame
    }

    /// Java: `handleCommand(ReceivedCommand)`.
    ///
    /// ```java
    /// InternalServerCommandDeleteGame deleteGameCommand = (InternalServerCommandDeleteGame) pReceivedCommand.getCommand();
    /// getServer().getGameCache().queueDbDelete(deleteGameCommand.getGameId(), deleteGameCommand.isWithGamesInfo());
    /// getServer().getDebugLog().log(IServerLogLevel.WARN, deleteGameCommand.getGameId(), "GameState deleted from db");
    /// return true;
    /// ```
    ///
    /// `GameCache.queueDbDelete(gameId, withGamesInfo)` (Java) builds a `DbTransaction` with
    /// (conditionally) a `DbGamesInfoDeleteParameter` plus an unconditional
    /// `DbGamesSerializedDeleteParameter`, then hands it to the async `DbUpdater`. Here the
    /// same delete pair runs inline against a connection from the shared
    /// `DbConnectionManager` pool (see `db_connection_manager.rs::pool_ready` for why this
    /// degrades to a no-op when no DB is configured, e.g. in tests).
    pub async fn handle_command(&self, delete_game_command: &InternalServerCommandDeleteGame) -> bool {
        let game_id = delete_game_command.get_game_id();
        let with_games_info = delete_game_command.is_with_games_info();
        if game_id > 0 {
            // `DbConnectionManager` is cloned out from behind the `std::sync::Mutex` before any
            // `.await` (see that struct's own `Clone` doc comment) so this handler's future
            // stays `Send` — required since `handle_command` is reachable from
            // `ServerCommandHandlerFactory::handle_internal_command`, which runs inside
            // `tokio::spawn(dispatch_loop(...))`.
            let manager = self.db_connection_manager.lock().unwrap().clone();
            if manager.pool_ready() {
                if let Ok(mut conn) = manager.open_db_connection().await {
                    if with_games_info {
                        let _ = DbGamesInfoDelete::new().execute(&mut conn, game_id).await;
                    }
                    let _ = DbGamesSerializedDelete::new().execute(&mut conn, game_id).await;
                    let _ = manager.close_db_connection(conn).await;
                }
            }
        }
        log::warn!("game {}: GameState deleted from db", game_id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Arc<Mutex<DbConnectionManager>> {
        Arc::new(Mutex::new(DbConnectionManager::new()))
    }

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerDeleteGame::new(setup());
    }

    #[test]
    fn get_id_is_internal_server_delete_game() {
        let handler = ServerCommandHandlerDeleteGame::new(setup());
        assert_eq!(handler.get_id(), NetCommandId::InternalServerDeleteGame);
    }

    /// Without a live DB pool configured, the DB delete step is skipped (`pool_ready()` is
    /// false) — the handler still extracts the command fields and returns `true`.
    #[tokio::test]
    async fn handle_command_without_db_pool_is_a_noop_returning_true() {
        let handler = ServerCommandHandlerDeleteGame::new(setup());
        let cmd = InternalServerCommandDeleteGame::new(42, true);
        assert!(handler.handle_command(&cmd).await);
    }

    #[tokio::test]
    async fn zero_game_id_is_a_noop_returning_true() {
        let handler = ServerCommandHandlerDeleteGame::new(setup());
        let cmd = InternalServerCommandDeleteGame::new(0, true);
        assert!(handler.handle_command(&cmd).await);
    }

    #[test]
    fn without_games_info_flag_is_read() {
        let cmd = InternalServerCommandDeleteGame::new(7, false);
        assert_eq!(cmd.get_game_id(), 7);
        assert!(!cmd.is_with_games_info());
    }
}
