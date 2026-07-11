//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSetPreventSketching`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSetPreventSketching;

impl ClientCommandHandlerSetPreventSketching {
    pub fn new() -> Self {
        Self
    }

    /// Java:
    /// ```java
    /// String prefix;
    /// if (getClient().getParameters().getCoach().equals(command.getCoach())) { prefix = "You are"; }
    /// else { prefix = "Coach " + command.getCoach() + " is"; }
    /// String action = command.isPreventSketching() ? "blocked" : "unblocked";
    /// ```
    /// `ClientParameters` is a GUI stub, so the local coach name is taken as an
    /// explicit parameter instead of `getClient().getParameters().getCoach()`.
    pub fn status_message(local_coach: &str, command_coach: &str, prevent_sketching: bool) -> String {
        let prefix = if local_coach == command_coach {
            "You are".to_string()
        } else {
            format!("Coach {} is", command_coach)
        };
        let action = if prevent_sketching { "blocked" } else { "unblocked" };
        format!("{} {}", prefix, action)
    }
}

impl ClientCommandHandler for ClientCommandHandlerSetPreventSketching {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSetPreventSketching
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`. Java returns
    /// `false` unconditionally (waits for `ClientCommandHandlerFactory` to be notified).
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, _mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerSetPreventSketching(command) = net_command {
            // java: ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
            // java: if (command.isPreventSketching()) { sketchManager.preventedFromSketching(...); }
            // java: else { sketchManager.allowSketching(...); }
            // java: ChatComponent chat = getClient().getUserInterface().getChat();
            // java: chat.append(TextStyle.SPECTATOR, prefix + " " + action);
            // java: SketchState sketchState = new SketchState(sketchManager.getAllSketches());
            // java: getClient().getGame().notifyObservers(new ModelChange(SKETCH_UPDATE, null, sketchState));
            // java: getClient().getUserInterface().getGameMenuBar().updateJoinedCoachesMenu();
            let _ = command.is_prevent_sketching();
            let _ = command.get_coach();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_set_prevent_sketching::ServerCommandSetPreventSketching;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn get_id_is_server_set_prevent_sketching() {
        assert_eq!(
            ClientCommandHandlerSetPreventSketching::new().get_id(),
            NetCommandId::ServerSetPreventSketching
        );
    }

    #[test]
    fn status_message_for_local_coach() {
        assert_eq!(
            ClientCommandHandlerSetPreventSketching::status_message("Alice", "Alice", true),
            "You are blocked"
        );
    }

    #[test]
    fn status_message_for_other_coach() {
        assert_eq!(
            ClientCommandHandlerSetPreventSketching::status_message("Alice", "Bob", false),
            "Coach Bob is unblocked"
        );
    }

    #[test]
    fn handle_net_command_always_returns_false_for_matching_command() {
        let mut handler = ClientCommandHandlerSetPreventSketching::new();
        let cmd = AnyServerCommand::ServerSetPreventSketching(ServerCommandSetPreventSketching::new("Alice", true));
        assert!(!handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_false_for_a_mismatched_command_type_too() {
        // Java always returns false regardless of the (unchecked) cast succeeding;
        // the Rust no-op match preserves the same return value.
        let mut handler = ClientCommandHandlerSetPreventSketching::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(!handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
