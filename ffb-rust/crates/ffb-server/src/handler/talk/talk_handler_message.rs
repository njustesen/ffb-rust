/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMessage.
/// Handles /message command — broadcasts admin message (DEV privilege, TEST_SERVER env).
use std::collections::HashSet;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

const MESSAGE_COMMAND: &str = "/message";

pub struct TalkHandlerMessage {
    base: TalkHandler,
}

impl TalkHandlerMessage {
    /// Java: `TalkHandlerMessage()` — `super(MESSAGE_COMMAND, 0, Client.PLAYER,
    /// Environment.TEST_SERVER, Privilege.DEV)`, which resolves to the `IdentityCommandAdapter`
    /// via `TalkHandler`'s no-adapter constructor overload.
    pub fn new() -> Self {
        let adapter = IdentityCommandAdapter::new();
        let mut commands = HashSet::new();
        commands.insert(MESSAGE_COMMAND.to_string());
        let commands = adapter.decorate_commands(commands);
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::Dev);
        Self {
            base: TalkHandler::new(commands, 0, Client::Player, Environment::TestServer, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` — sends
    /// admin message. Returns the message Java would have sent via
    /// `communication.sendAdminMessage(new String[]{...})` (see `talk_handler.rs` doc for
    /// why the outbound send is adapted to a returned value).
    pub fn handle(&self, commands: &[String]) -> Vec<String> {
        let message = commands.join(" ");
        if message.len() > MESSAGE_COMMAND.len() {
            vec![message[MESSAGE_COMMAND.len() + 1..].trim().to_string()]
        } else {
            Vec::new()
        }
    }
}

impl Default for TalkHandlerMessage {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() { let _ = TalkHandlerMessage::new(); }

    #[test]
    fn handle_returns_message_text_after_command() {
        let h = TalkHandlerMessage::new();
        let commands = vec!["/message".to_string(), "Hello".to_string(), "world".to_string()];
        let info = h.handle(&commands);
        assert_eq!(info, vec!["Hello world".to_string()]);
    }

    #[test]
    fn handle_returns_empty_when_no_message_present() {
        let h = TalkHandlerMessage::new();
        let commands = vec!["/message".to_string()];
        let info = h.handle(&commands);
        assert!(info.is_empty());
    }

    #[test]
    fn handle_trims_surrounding_whitespace() {
        let h = TalkHandlerMessage::new();
        let commands = vec!["/message".to_string(), " padded ".to_string()];
        let info = h.handle(&commands);
        assert_eq!(info, vec!["padded".to_string()]);
    }
}
