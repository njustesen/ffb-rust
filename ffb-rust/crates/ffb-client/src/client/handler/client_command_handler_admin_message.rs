//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerAdminMessage`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: `implements IDialogCloseListener`. `DialogInformation`/`IDialog` have no
/// Rust equivalent yet, so the dialog show/close cycle is left as `// java:` notes.
#[derive(Debug, Default)]
pub struct ClientCommandHandlerAdminMessage;

impl ClientCommandHandlerAdminMessage {
    pub fn new() -> Self {
        Self
    }

    /// Java: `dialogClosed(IDialog pDialog)`.
    pub fn dialog_closed(&self) {
        // java: pDialog.hideDialog();
    }
}

impl ClientCommandHandler for ClientCommandHandlerAdminMessage {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerAdminMessage
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, _mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerAdminMessage(message_command) = net_command {
            // java: DialogInformation messageDialog = new DialogInformation(getClient(),
            // java:   "Administrator Message", messageCommand.getMessages(), DialogInformation.OK_DIALOG, false);
            // java: messageDialog.showDialog(this);
            let _ = message_command.get_messages();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_admin_message::ServerCommandAdminMessage;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn get_id_is_server_admin_message() {
        assert_eq!(ClientCommandHandlerAdminMessage::new().get_id(), NetCommandId::ServerAdminMessage);
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerAdminMessage::new();
        let cmd = AnyServerCommand::ServerAdminMessage(ServerCommandAdminMessage::new(vec!["hi".into()]));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_across_modes() {
        let mut handler = ClientCommandHandlerAdminMessage::new();
        let cmd = AnyServerCommand::ServerAdminMessage(ServerCommandAdminMessage::new(vec![]));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerAdminMessage::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn dialog_closed_does_not_panic() {
        ClientCommandHandlerAdminMessage::new().dialog_closed();
    }

    #[test]
    fn default_constructs_a_handler() {
        let handler = ClientCommandHandlerAdminMessage::default();
        assert_eq!(handler.get_id(), NetCommandId::ServerAdminMessage);
    }
}
