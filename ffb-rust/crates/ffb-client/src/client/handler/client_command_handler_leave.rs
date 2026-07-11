//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerLeave`.

use ffb_model::enums::NetCommandId;
use ffb_model::model::client_mode::ClientMode;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerLeave;

impl ClientCommandHandlerLeave {
    pub fn new() -> Self {
        Self
    }

    /// Java: `ClientMode.PLAYER == leaveCommand.getClientMode()` — the pure condition
    /// under which `getClient().getClientData().setTurnTimerStopped(true)` would fire.
    /// `ClientData` is still a GUI stub with no fields, so the mutation itself is not
    /// performed here.
    pub fn should_stop_turn_timer(client_mode: ClientMode) -> bool {
        client_mode == ClientMode::PLAYER
    }
}

impl ClientCommandHandler for ClientCommandHandlerLeave {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerLeave
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        if mode == ClientCommandHandlerMode::QUEUING {
            return true;
        }

        if let AnyServerCommand::ServerLeave(leave_command) = net_command {
            if Self::should_stop_turn_timer(leave_command.get_client_mode()) {
                // java: getClient().getClientData().setTurnTimerStopped(true);
            }

            // java: getClient().getClientData().setSpectatorCount(leaveCommand.getSpectatorCount());
            // java: getClient().getClientData().setSpectators(leaveCommand.getSpectators());
            let _ = leave_command.get_spectator_count();
            let _ = leave_command.get_spectators();

            if mode != ClientCommandHandlerMode::REPLAYING {
                // java: UserInterface userInterface = getClient().getUserInterface();
                // java: userInterface.getLog().markCommandBegin(leaveCommand.getCommandNr());
                // java: userInterface.getStatusReport().reportLeave(leaveCommand);
                // java: userInterface.getLog().markCommandEnd(leaveCommand.getCommandNr());
                // java: refreshSideBars();
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_leave::ServerCommandLeave;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn get_id_is_server_leave() {
        assert_eq!(ClientCommandHandlerLeave::new().get_id(), NetCommandId::ServerLeave);
    }

    #[test]
    fn should_stop_turn_timer_true_for_player_mode() {
        assert!(ClientCommandHandlerLeave::should_stop_turn_timer(ClientMode::PLAYER));
    }

    #[test]
    fn should_stop_turn_timer_false_for_spectator_and_replay() {
        assert!(!ClientCommandHandlerLeave::should_stop_turn_timer(ClientMode::SPECTATOR));
        assert!(!ClientCommandHandlerLeave::should_stop_turn_timer(ClientMode::REPLAY));
    }

    #[test]
    fn handle_net_command_short_circuits_when_queuing() {
        let mut handler = ClientCommandHandlerLeave::new();
        let cmd = AnyServerCommand::ServerLeave(ServerCommandLeave::new("Bob", ClientMode::PLAYER, vec![]));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerLeave::new();
        let cmd = AnyServerCommand::ServerLeave(ServerCommandLeave::new("Bob", ClientMode::SPECTATOR, vec!["s1".into()]));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerLeave::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
