//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerJoin`.

use ffb_model::enums::NetCommandId;
use ffb_model::model::client_mode::ClientMode;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: computed `GameTitle` fields (`homeCoach`/`awayCoach`) — pure logic extracted
/// from `handleNetCommand`'s player-name reordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeAwayCoaches {
    pub home_coach: String,
    pub away_coach: String,
}

#[derive(Debug, Default)]
pub struct ClientCommandHandlerJoin;

impl ClientCommandHandlerJoin {
    pub fn new() -> Self {
        Self
    }

    /// Java:
    /// ```java
    /// String[] players = joinCommand.getPlayerNames();
    /// if (ArrayTool.isProvided(players) && (players.length > 1)) {
    ///     String homeCoach; String awayCoach;
    ///     if (players[1].equals(coachName)) {
    ///         homeCoach = players[1]; awayCoach = players[0];
    ///     } else {
    ///         homeCoach = players[0]; awayCoach = players[1];
    ///     }
    ///     ...
    /// }
    /// ```
    /// Returns `None` when Java would skip building a `GameTitle` (fewer than 2 player names).
    pub fn home_away_coaches(player_names: &[String], coach_name: &str) -> Option<HomeAwayCoaches> {
        if player_names.len() <= 1 {
            return None;
        }
        let (home_coach, away_coach) = if player_names[1] == coach_name {
            (player_names[1].clone(), player_names[0].clone())
        } else {
            (player_names[0].clone(), player_names[1].clone())
        };
        Some(HomeAwayCoaches { home_coach, away_coach })
    }

    /// Java: `ClientMode.PLAYER == joinCommand.getClientMode()` — the pure condition
    /// under which `getClient().getClientData().setTurnTimerStopped(false)` would fire.
    pub fn should_resume_turn_timer(client_mode: ClientMode) -> bool {
        client_mode == ClientMode::PLAYER
    }

    /// Java: `pMode != REPLAYING && coachName != null && !coachName.equals(joinCommand.getCoach())`.
    pub fn should_report_join(mode: ClientCommandHandlerMode, coach_name: Option<&str>, joining_coach: &str) -> bool {
        mode != ClientCommandHandlerMode::REPLAYING
            && coach_name.is_some_and(|name| name != joining_coach)
    }
}

impl ClientCommandHandler for ClientCommandHandlerJoin {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerJoin
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        if mode == ClientCommandHandlerMode::QUEUING {
            return true;
        }

        if let AnyServerCommand::ServerJoin(join_command) = net_command {
            if Self::should_resume_turn_timer(join_command.get_client_mode()) {
                // java: getClient().getClientData().setTurnTimerStopped(false);
            }

            // java: String coachName = getClient().getParameters().getCoach();
            // `ClientParameters` is a GUI stub; the coach-name lookup can't be reached here,
            // so `home_away_coaches`/`should_report_join` are exposed for direct testing
            // with an explicit `coach_name` instead.
            if let Some(_coaches) = Self::home_away_coaches(join_command.get_player_names(), "") {
                // java: GameTitle gameTitle = new GameTitle(); ...; updateGameTitle(gameTitle);
            }

            // java: getClient().getClientData().setSpectatorCount(joinCommand.getSpectatorCount());
            // java: getClient().getClientData().setSpectators(joinCommand.getSpectators());
            let _ = join_command.get_spectator_count();
            let _ = join_command.get_spectators();

            // java: if (pMode != REPLAYING && coachName != null && !coachName.equals(joinCommand.getCoach())) { ... refreshSideBars(); }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_join::ServerCommandJoin;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn get_id_is_server_join() {
        assert_eq!(ClientCommandHandlerJoin::new().get_id(), NetCommandId::ServerJoin);
    }

    #[test]
    fn home_away_coaches_none_for_fewer_than_two_names() {
        assert!(ClientCommandHandlerJoin::home_away_coaches(&[], "Alice").is_none());
        assert!(ClientCommandHandlerJoin::home_away_coaches(&["Alice".into()], "Alice").is_none());
    }

    #[test]
    fn home_away_coaches_swaps_when_second_player_is_the_local_coach() {
        let names = vec!["Alice".to_string(), "Bob".to_string()];
        let result = ClientCommandHandlerJoin::home_away_coaches(&names, "Bob").unwrap();
        assert_eq!(result.home_coach, "Bob");
        assert_eq!(result.away_coach, "Alice");
    }

    #[test]
    fn home_away_coaches_keeps_order_otherwise() {
        let names = vec!["Alice".to_string(), "Bob".to_string()];
        let result = ClientCommandHandlerJoin::home_away_coaches(&names, "Someone Else").unwrap();
        assert_eq!(result.home_coach, "Alice");
        assert_eq!(result.away_coach, "Bob");
    }

    #[test]
    fn should_resume_turn_timer_only_for_player_mode() {
        assert!(ClientCommandHandlerJoin::should_resume_turn_timer(ClientMode::PLAYER));
        assert!(!ClientCommandHandlerJoin::should_resume_turn_timer(ClientMode::SPECTATOR));
    }

    #[test]
    fn should_report_join_false_when_replaying_or_same_coach() {
        assert!(!ClientCommandHandlerJoin::should_report_join(ClientCommandHandlerMode::REPLAYING, Some("Alice"), "Bob"));
        assert!(!ClientCommandHandlerJoin::should_report_join(ClientCommandHandlerMode::PLAYING, Some("Alice"), "Alice"));
        assert!(!ClientCommandHandlerJoin::should_report_join(ClientCommandHandlerMode::PLAYING, None, "Alice"));
    }

    #[test]
    fn should_report_join_true_for_other_coach_joining_while_playing() {
        assert!(ClientCommandHandlerJoin::should_report_join(ClientCommandHandlerMode::PLAYING, Some("Alice"), "Bob"));
    }

    #[test]
    fn handle_net_command_short_circuits_when_queuing() {
        let mut handler = ClientCommandHandlerJoin::new();
        let cmd = AnyServerCommand::ServerJoin(ServerCommandJoin::new("Bob", ClientMode::PLAYER, vec![], vec![], ""));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerJoin::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
