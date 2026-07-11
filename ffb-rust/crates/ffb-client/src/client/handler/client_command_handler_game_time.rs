//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerGameTime`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

/// Java: local (non-persisted) `GameTitle gameTitle = new GameTitle()` built inside
/// `handleNetCommand`, populated with `setGameTime`/`setTurnTime` and passed to
/// `updateGameTitle(gameTitle)`. `client/GameTitle.rs` is still a GUI-coupled stub
/// with no fields (title-bar rendering, out of scope), so this minimal struct
/// captures just the two values the Java method actually sets, in place of
/// inventing fields on the stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameTimeUpdate {
    pub game_time: i64,
    pub turn_time: i64,
}

#[derive(Debug, Default)]
pub struct ClientCommandHandlerGameTime {
    /// Not present in Java — records the most recent `GameTitle` update so
    /// pure-logic tests can assert on it without a real `UserInterface`.
    pub last_update: Option<GameTimeUpdate>,
}

impl ClientCommandHandlerGameTime {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ClientCommandHandler for ClientCommandHandlerGameTime {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerGameTime
    }

    /// Java:
    /// ```java
    /// ServerCommandGameTime gameTimeCommand = (ServerCommandGameTime) netCommand;
    /// GameTitle gameTitle = new GameTitle();
    /// gameTitle.setGameTime(gameTimeCommand.getGameTime());
    /// gameTitle.setTurnTime(gameTimeCommand.getTurnTime());
    /// updateGameTitle(gameTitle);
    /// return true;
    /// ```
    fn handle_net_command(
        &mut self,
        net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        if let AnyServerCommand::ServerGameTime(game_time_command) = net_command {
            let update = GameTimeUpdate {
                game_time: game_time_command.get_game_time(),
                turn_time: game_time_command.get_turn_time(),
            };
            self.last_update = Some(update);
            // java: updateGameTitle(gameTitle); — invokeLater(GameTitleUpdateTask), GUI side effect.
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::commands::server_command_game_time::ServerCommandGameTime;

    #[test]
    fn get_id_is_server_game_time() {
        assert_eq!(ClientCommandHandlerGameTime::new().get_id(), NetCommandId::ServerGameTime);
    }

    #[test]
    fn handle_net_command_records_game_and_turn_time() {
        let mut handler = ClientCommandHandlerGameTime::new();
        let cmd = AnyServerCommand::ServerGameTime(ServerCommandGameTime::new(60_000, 30_000));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(
            handler.last_update,
            Some(GameTimeUpdate { game_time: 60_000, turn_time: 30_000 })
        );
    }

    #[test]
    fn handle_net_command_ignores_mismatched_command_type() {
        let mut handler = ClientCommandHandlerGameTime::new();
        let cmd = AnyServerCommand::ServerClearSketches(
            ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches::new(),
        );
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(handler.last_update, None);
    }

    #[test]
    fn handle_net_command_returns_true_across_all_modes() {
        let mut handler = ClientCommandHandlerGameTime::new();
        let cmd = AnyServerCommand::ServerGameTime(ServerCommandGameTime::new(1, 2));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn default_constructs_a_handler_with_no_update_yet() {
        let handler = ClientCommandHandlerGameTime::default();
        assert_eq!(handler.last_update, None);
    }
}
