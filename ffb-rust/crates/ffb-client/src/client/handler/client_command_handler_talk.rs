//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerTalk`.
//!
//! Java's `TextStyle` enum has no Rust translation yet, so the computed style
//! selection is represented as plain string tags (`"HOME"`, `"AWAY"`, `"ADMIN"`,
//! `"DEV"`, `"SPECTATOR"`, `"NONE"`) instead of inventing a `TextStyle` type.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: `TextStyle style` / `TextStyle prefixStyle` computed per talk line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkStyle {
    None,
    Home,
    Away,
    Admin,
    Dev,
    Spectator,
}

#[derive(Debug, Default)]
pub struct ClientCommandHandlerTalk;

impl ClientCommandHandlerTalk {
    pub fn new() -> Self {
        Self
    }

    /// Java:
    /// ```java
    /// if (StringTool.isProvided(coach)) {
    ///     ...
    ///     if (coach.equals(game.getTeamHome().getCoach())) { style = HOME; prefixStyle = HOME_BOLD; }
    ///     else if (coach.equals(game.getTeamAway().getCoach())) { style = AWAY; prefixStyle = AWAY_BOLD; }
    ///     else if (talkCommand.getMode() == Mode.STAFF) { style = ADMIN; prefixStyle = ADMIN_BOLD; }
    ///     else if (talkCommand.getMode() == Mode.DEV) { style = DEV; prefixStyle = DEV_BOLD; }
    ///     else { style = SPECTATOR; prefixStyle = SPECTATOR_BOLD; }
    /// }
    /// ```
    /// Returns `None` when Java leaves `style`/`prefixStyle` at their `TextStyle.NONE` default
    /// (i.e. `coach` is empty).
    pub fn talk_style(coach: &str, mode: &str, home_coach: &str, away_coach: &str) -> TalkStyle {
        if coach.is_empty() {
            return TalkStyle::None;
        }
        if coach == home_coach {
            TalkStyle::Home
        } else if coach == away_coach {
            TalkStyle::Away
        } else if mode == "STAFF" {
            TalkStyle::Admin
        } else if mode == "DEV" {
            TalkStyle::Dev
        } else {
            TalkStyle::Spectator
        }
    }

    /// Java: `status.append("<").append(mode.getPrefix()).append(coach).append("> ")`,
    /// only built when `StringTool.isProvided(coach)`. `Mode.getPrefix()` has no Rust
    /// translation of the `Mode` enum here (mode is carried as a plain string on
    /// `ServerCommandTalk`), so the prefix is passed in explicitly.
    pub fn status_prefix(coach: &str, mode_prefix: &str) -> String {
        if coach.is_empty() {
            String::new()
        } else {
            format!("<{}{}> ", mode_prefix, coach)
        }
    }
}

impl ClientCommandHandler for ClientCommandHandlerTalk {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerTalk
    }

    /// Java: `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(&mut self, net_command: &AnyServerCommand, _mode: ClientCommandHandlerMode) -> bool {
        if let AnyServerCommand::ServerTalk(talk_command) = net_command {
            // java: Game game = getClient().getGame(); String coach = talkCommand.getCoach();
            // `game.getTeamHome()/getTeamAway()` coach names aren't reachable without a live
            // Game, so per-line style/prefix computation is exposed via `talk_style`/
            // `status_prefix` for direct testing instead of being invented here.
            for _talk in talk_command.get_talks() {
                // java: ChatComponent chat = getClient().getUserInterface().getChat();
                // java: chat.parseAndAppend(style, prefixStyle, prefix, talk);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;
    use ffb_protocol::commands::server_command_talk::ServerCommandTalk;

    #[test]
    fn get_id_is_server_talk() {
        assert_eq!(ClientCommandHandlerTalk::new().get_id(), NetCommandId::ServerTalk);
    }

    #[test]
    fn talk_style_none_for_empty_coach() {
        assert_eq!(ClientCommandHandlerTalk::talk_style("", "REGULAR", "Alice", "Bob"), TalkStyle::None);
    }

    #[test]
    fn talk_style_home_and_away() {
        assert_eq!(ClientCommandHandlerTalk::talk_style("Alice", "REGULAR", "Alice", "Bob"), TalkStyle::Home);
        assert_eq!(ClientCommandHandlerTalk::talk_style("Bob", "REGULAR", "Alice", "Bob"), TalkStyle::Away);
    }

    #[test]
    fn talk_style_staff_and_dev() {
        assert_eq!(ClientCommandHandlerTalk::talk_style("Admin", "STAFF", "Alice", "Bob"), TalkStyle::Admin);
        assert_eq!(ClientCommandHandlerTalk::talk_style("Dev", "DEV", "Alice", "Bob"), TalkStyle::Dev);
    }

    #[test]
    fn talk_style_defaults_to_spectator() {
        assert_eq!(ClientCommandHandlerTalk::talk_style("Watcher", "REGULAR", "Alice", "Bob"), TalkStyle::Spectator);
    }

    #[test]
    fn status_prefix_builds_expected_string() {
        assert_eq!(ClientCommandHandlerTalk::status_prefix("Alice", "@"), "<@Alice> ");
        assert_eq!(ClientCommandHandlerTalk::status_prefix("", "@"), "");
    }

    #[test]
    fn handle_net_command_returns_true_for_matching_command() {
        let mut handler = ClientCommandHandlerTalk::new();
        let cmd = AnyServerCommand::ServerTalk(ServerCommandTalk::new("Alice", vec!["hi".into()], "REGULAR"));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_is_a_no_op_for_a_mismatched_command_type() {
        let mut handler = ClientCommandHandlerTalk::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }
}
