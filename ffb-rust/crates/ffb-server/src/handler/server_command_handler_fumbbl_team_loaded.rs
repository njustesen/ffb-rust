/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerFumbblTeamLoaded.
use ffb_model::enums::NetCommandId;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_fumbbl_team_loaded::InternalServerCommandFumbblTeamLoaded;

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
    /// `UtilServerStartGame::join_game_as_player_and_check_if_ready_to_start` is explicitly
    /// unported in the Rust engine (see `ffb_engine::util::util_server_start_game`, which
    /// documents it as skipped: it touches the DB, the WebSocket session, `SessionManager`, and
    /// the step `SequenceGenerator` — none of which have a session-aware equivalent reachable
    /// from here yet). The `GameCache` lookup and the null-gamestate short-circuit are ported
    /// for real below; the join/ready-check dispatch is the single narrowly-gated remainder.
    pub fn handle_command(
        &self,
        command: &InternalServerCommandFumbblTeamLoaded,
        game_cache: &GameCache,
    ) -> bool {
        let game_state = match game_cache.get_game_state_by_id(command.get_game_id()) {
            Some(gs) => gs,
            None => return false,
        };

        let _ = game_state;
        let _ = command.get_coach();
        let _ = command.is_home_team();
        let _ = command.get_account_properties();
        todo!(
            "Phase ZV: needs UtilServerStartGame::join_game_as_player_and_check_if_ready_to_start \
             (+ FumbblRequestCheckGamestate enqueue), not yet wired"
        )
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

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerFumbblTeamLoaded::new();
    }

    #[test]
    fn get_id_is_fumbbl_team_loaded() {
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        assert_eq!(h.get_id(), NetCommandId::InternalServerFumbblTeamLoaded);
    }

    #[test]
    fn handle_command_missing_gamestate_returns_false() {
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        let cache = GameCache::new();
        let command = InternalServerCommandFumbblTeamLoaded::new(999, "coach".into(), true, vec![]);
        assert!(!h.handle_command(&command, &cache));
    }

    #[test]
    fn handle_command_missing_gamestate_does_not_reach_join_dispatch() {
        // A game id that was never created in the cache must short-circuit before the
        // (currently unwired) join/ready-check dispatch, for any coach/home-team combination.
        let h = ServerCommandHandlerFumbblTeamLoaded::new();
        let cache = GameCache::new();
        let command = InternalServerCommandFumbblTeamLoaded::new(
            424242,
            "AwayCoach".into(),
            false,
            vec!["DEV".into()],
        );
        assert!(!h.handle_command(&command, &cache));
    }
}
