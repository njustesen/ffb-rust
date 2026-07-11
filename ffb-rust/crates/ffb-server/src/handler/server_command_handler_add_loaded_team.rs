/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerAddLoadedTeam.
use ffb_model::enums::{GameStatus, NetCommandId};
use ffb_model::model::team::Team;
use crate::game_cache::GameCache;
use crate::net::commands::internal_server_command::InternalServerCommand;
use crate::net::commands::internal_server_command_add_loaded_team::InternalServerCommandAddLoadedTeam;

pub struct ServerCommandHandlerAddLoadedTeam;

impl ServerCommandHandlerAddLoadedTeam {
    pub fn new() -> Self {
        Self
    }

    /// Java: getId() — returns NetCommandId for ADD_LOADED_TEAM.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalServerAddLoadedTeam
    }

    /// Java: handleCommand(ReceivedCommand) — handles adding a loaded team to the game.
    ///
    /// `team` corresponds to `command.getTeam()`. The Rust
    /// `InternalServerCommandAddLoadedTeam` does not yet carry a typed `Team` field
    /// (that command struct predates the `Team` model port), so it is passed in
    /// explicitly here — this preserves the real 1:1 business logic below rather
    /// than inventing a substitute.
    ///
    /// `GameCache::add_team_to_game` (added in Phase ZX.2) and `Game::status` (a
    /// `GameStatus`, unlike Java where status lives on the server-side `GameState`
    /// wrapper instead of `Game` itself) now exist, so the real business logic —
    /// resolving `homeTeam`, swapping the team into the game/field model, and the
    /// `GameStatus::Scheduled` branch (log "GAME SCHEDULED") — is ported for real below.
    /// What remains unported is Java's `else` tail: dispatching a fresh
    /// `InternalServerCommandFumbblTeamLoaded` back through
    /// `getServer().getCommunication().handleCommand(...)`. This whole `handler/` module
    /// family is not yet wired into the live `ServerCommunication` dispatch loop at all
    /// (see `ServerCommandHandlerFactory`'s own "Known gap" doc comment — its
    /// `handle_command` only recognizes `ffb_protocol::client_commands::ClientCommand`,
    /// which has no internal-command variants), so there is no dispatch sink to hand the
    /// built `InternalServerCommandFumbblTeamLoaded` off to yet; that remains a narrow,
    /// clearly-scoped gap rather than a fabricated stand-in.
    pub fn handle_command(
        &self,
        command: &InternalServerCommandAddLoadedTeam,
        team: Team,
        game_cache: &mut GameCache,
    ) -> bool {
        let game_id = command.get_game_id();

        let game_state = match game_cache.get_game_state_by_id_mut(game_id) {
            Some(gs) => gs,
            None => {
                // Java: getServer().getDebugLog().log(ERROR, command.getGameId(), "No gamestate
                // found in command or cache, should only happen if command was created during
                // scheduling a game and has been serialized");
                log::error!(
                    "game {}: no gamestate found in command or cache, should only happen if \
                     command was created during scheduling a game and has been serialized",
                    game_id
                );
                return true;
            }
        };

        let existing_home_team_id = match game_state.get_game() {
            Some(g) => g.team_home.id.clone(),
            None => {
                log::error!(
                    "game {}: no gamestate found in command or cache, should only happen if \
                     command was created during scheduling a game and has been serialized",
                    game_id
                );
                return true;
            }
        };

        // Java: game.teamsAreSkeletons(); — no-op marker call in the Java model; the Rust
        // `Game`/`Team` model has no skeleton/inflated distinction, so there is nothing to mark.

        // Java: Boolean homeTeam = command.getHomeTeam();
        //       if (homeTeam == null) homeTeam = (!StringTool.isProvided(game.getTeamHome().getId())
        //           || team.getId().equals(game.getTeamHome().getId()));
        let home_team = command
            .get_home_team()
            .unwrap_or_else(|| resolve_home_team(&existing_home_team_id, &team.id));

        // Java: getServer().getGameCache().addTeamToGame(gameState, team, homeTeam);
        let game = game_state.get_game_mut().expect("checked above");
        GameCache::add_team_to_game(game, team, home_team);

        if GameStatus::Scheduled == game.status {
            // Java: if (StringTool.isProvided(game.getTeamHome().getId()) &&
            //           StringTool.isProvided(game.getTeamAway().getId())) { ... log GAME SCHEDULED ... }
            if !game.team_home.id.is_empty() && !game.team_away.id.is_empty() {
                log::warn!(
                    "GAME SCHEDULED {:?} vs. {:?} (game {})",
                    game.team_home.name,
                    game.team_away.name,
                    game_id
                );
            }
        } else {
            // Java: `getServer().getCommunication().handleCommand(new ReceivedCommand(
            //           new InternalServerCommandFumbblTeamLoaded(gameState.getId(), coach, homeTeam,
            //           command.getAccountProperties()), session))`.
            // No internal-command redispatch sink exists in this crate yet (see doc comment above).
            log::debug!(
                "game {}: team loaded for coach {} (home={}) — InternalServerCommandFumbblTeamLoaded \
                 redispatch not wired (no internal-command dispatch sink exists yet)",
                game_id,
                command.get_coach(),
                home_team
            );
        }

        true
    }
}

/// Java: the null-`homeTeam` inference inside `handleCommand`:
/// `!StringTool.isProvided(game.getTeamHome().getId()) || team.getId().equals(game.getTeamHome().getId())`.
fn resolve_home_team(existing_home_team_id: &str, team_id: &str) -> bool {
    existing_home_team_id.is_empty() || team_id == existing_home_team_id
}

impl Default for ServerCommandHandlerAddLoadedTeam {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(id: &str) -> Team {
        Team {
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

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerAddLoadedTeam::new();
    }

    #[test]
    fn get_id_is_internal_server_add_loaded_team() {
        let h = ServerCommandHandlerAddLoadedTeam::new();
        assert_eq!(h.get_id(), NetCommandId::InternalServerAddLoadedTeam);
    }

    #[test]
    fn handle_command_missing_gamestate_returns_true() {
        let h = ServerCommandHandlerAddLoadedTeam::new();
        let mut cache = GameCache::new();
        let command = InternalServerCommandAddLoadedTeam::new(999, "coach".into(), None, vec![]);
        let t = team("t1");
        assert!(h.handle_command(&command, t, &mut cache));
    }

    #[test]
    fn handle_command_unstarted_game_returns_true() {
        // GameState exists in the cache but has no driver/Game yet (mirrors the Java case of a
        // serialized command referencing a gamestate that was never fully created).
        let h = ServerCommandHandlerAddLoadedTeam::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        let command = InternalServerCommandAddLoadedTeam::new(game_id, "coach".into(), None, vec![]);
        let t = team("t1");
        assert!(h.handle_command(&command, t, &mut cache));
    }

    #[test]
    fn handle_command_scheduled_status_adds_team_and_logs_scheduled() {
        use ffb_model::enums::Rules;

        let h = ServerCommandHandlerAddLoadedTeam::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        {
            let gs = cache.get_game_state_by_id_mut(game_id).unwrap();
            let mut home_placeholder = team("");
            home_placeholder.id = String::new();
            gs.start_game(home_placeholder, team("away1"), Rules::Bb2025, 0);
            gs.get_game_mut().unwrap().status = GameStatus::Scheduled;
        }
        let command = InternalServerCommandAddLoadedTeam::new(game_id, "coach".into(), None, vec![]);
        let t = team("home1");
        assert!(h.handle_command(&command, t, &mut cache));

        let game = cache.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        assert_eq!(game.team_home.id, "home1");
        assert_eq!(game.status, GameStatus::Scheduled);
    }

    #[test]
    fn handle_command_non_scheduled_status_still_adds_team() {
        use ffb_model::enums::Rules;

        let h = ServerCommandHandlerAddLoadedTeam::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        {
            let gs = cache.get_game_state_by_id_mut(game_id).unwrap();
            gs.start_game(team("oldhome"), team("away1"), Rules::Bb2025, 0);
            // Default status from `GameState::start_game` is not `Scheduled`, so this
            // exercises the (currently unwired) redispatch branch instead of the log branch.
        }
        let command = InternalServerCommandAddLoadedTeam::new(game_id, "coach".into(), Some(true), vec![]);
        let t = team("newhome");
        assert!(h.handle_command(&command, t, &mut cache));

        let game = cache.get_game_state_by_id(game_id).unwrap().get_game().unwrap();
        assert_eq!(game.team_home.id, "newhome");
    }

    #[test]
    fn resolve_home_team_empty_existing_id_is_home() {
        assert!(resolve_home_team("", "away-team"));
    }

    #[test]
    fn resolve_home_team_matching_id_is_home() {
        assert!(resolve_home_team("t1", "t1"));
    }

    #[test]
    fn resolve_home_team_non_matching_id_is_not_home() {
        assert!(!resolve_home_team("t1", "t2"));
    }
}
