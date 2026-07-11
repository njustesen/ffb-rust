/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFumbblTeamLoaded.
use std::sync::Mutex;

use tokio::sync::mpsc;

use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use crate::db::db_connection_manager::DbConnectionManager;
use crate::model::received_command::SessionId;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_fumbbl_team_loaded::InternalServerCommandFumbblTeamLoaded;
use crate::net::session_manager::SessionManager;
use crate::util::server_start_game::{self, MarkerContext};

pub struct ServerCommandHandlerFumbblTeamLoaded;

impl ServerCommandHandlerFumbblTeamLoaded {
    pub fn new() -> Self {
        Self
    }

    /// Java: getId() — returns NetCommandId for FUMBBL_TEAM_LOADED.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerFumbblTeamLoaded
    }

    /// Java: handleCommand(ReceivedCommand) — handles a FUMBBL team-loaded event.
    ///
    /// Java:
    /// ```java
    /// GameState gameState = getServer().getGameCache().getGameStateById(teamLoadedCommand.getGameId());
    /// if (gameState == null) return false;
    /// if (UtilServerStartGame.joinGameAsPlayerAndCheckIfReadyToStart(gameState, pReceivedCommand.getSession(),
    ///         teamLoadedCommand.getCoach(), teamLoadedCommand.isHomeTeam(), teamLoadedCommand.getAccountProperties())) {
    ///     getServer().getRequestProcessor().add(new FumbblRequestCheckGamestate(gameState));
    /// }
    /// return true;
    /// ```
    ///
    /// `UtilServerStartGame::join_game_as_player_and_check_if_ready_to_start` is now ported
    /// for real (Phase ZX.3, `util::server_start_game`), so this handler calls it for real
    /// below. `session_id`/`sender` stand in for Java's `pReceivedCommand.getSession()`
    /// (per this crate's convention of threading the session explicitly rather than a
    /// Jetty `Session` object — see `server_command_handler_join_approved.rs`).
    /// `FumbblRequestCheckGamestate` has no Rust translation anywhere in this crate, so
    /// that enqueue remains a documented, logged gap rather than a fabricated stand-in.
    ///
    /// Takes the already-looked-up `game` (an owned clone, `Game: Clone`) rather than a
    /// `&GameCache` to look it up from directly: `GameCache` holds `Box<dyn Step>` engine
    /// state that isn't `Sync`, so a `&GameCache` (or a `std::sync::MutexGuard` around one)
    /// can't be held across this method's own `.await` below without making the caller's
    /// future non-`Send` — a hard blocker for `ServerCommandHandlerFactory::handle_internal_command`,
    /// which runs inside `tokio::spawn(dispatch_loop(...))`. Callers instead lock `GameCache`,
    /// clone out the one `Game` needed, and drop the lock before calling this.
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_command(
        &self,
        command: &InternalServerCommandFumbblTeamLoaded,
        game: Option<&Game>,
        session_manager: &Mutex<SessionManager>,
        session_id: SessionId,
        sender: mpsc::UnboundedSender<String>,
        db: &DbConnectionManager,
        client_properties: &[(String, String)],
        marker_ctx: Option<MarkerContext<'_>>,
    ) -> bool {
        let game = match game {
            Some(g) => g,
            None => return false,
        };

        let ready_to_start = server_start_game::join_game_as_player_and_check_if_ready_to_start(
            game,
            command.get_game_id(),
            session_manager,
            session_id,
            command.get_coach().to_string(),
            command.is_home_team(),
            command.get_account_properties().to_vec(),
            sender,
            db,
            client_properties,
            marker_ctx,
        )
        .await;

        if ready_to_start {
            // Java: `getServer().getRequestProcessor().add(new FumbblRequestCheckGamestate(gameState))`.
            // No `FumbblRequestCheckGamestate` exists in this crate — documented gap.
            log::debug!(
                "game {}: ready to start — FumbblRequestCheckGamestate not ported",
                command.get_game_id()
            );
        }

        true
    }
}

impl Default for ServerCommandHandlerFumbblTeamLoaded {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use crate::game_cache::GameCache;

    fn team(id: &str, coach: &str) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {}", id),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: coach.into(),
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

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerFumbblTeamLoaded::new();
    }

    #[test]
    fn get_id_is_fumbbl_team_loaded() {
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        assert_eq!(h.get_id(), NetCommandId::InternalServerFumbblTeamLoaded);
    }

    #[tokio::test]
    async fn handle_command_missing_gamestate_returns_false() {
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let command = InternalServerCommandFumbblTeamLoaded::new(999, "coach".into(), true, vec![]);
        assert!(!h.handle_command(&command, None, &sm, 1, tx, &db, &[], None).await);
    }

    #[tokio::test]
    async fn handle_command_missing_gamestate_does_not_reach_join_dispatch() {
        // A missing `game` (no gamestate found in the cache) must short-circuit before the
        // join/ready-check dispatch, for any coach/home-team combination.
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let command = InternalServerCommandFumbblTeamLoaded::new(
            424242,
            "AwayCoach".into(),
            false,
            vec!["DEV".into()],
        );
        assert!(!h.handle_command(&command, None, &sm, 1, tx, &db, &[], None).await);
    }

    #[tokio::test]
    async fn handle_command_registers_the_joining_session() {
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home", "Home"), team("away", "Away"), Rules::Bb2025, 0);
        let game = cache.get_game_state_by_id(game_id).unwrap().get_game().cloned();
        let sm = Mutex::new(SessionManager::new());
        let db = DbConnectionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let command = InternalServerCommandFumbblTeamLoaded::new(game_id, "Home".into(), true, vec![]);

        let result = h.handle_command(&command, game.as_ref(), &sm, 1, tx, &db, &[], None).await;
        assert!(result);
        assert_eq!(sm.lock().unwrap().get_coach_for_session(1), Some("Home"));
    }
}
