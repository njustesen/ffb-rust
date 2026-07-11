//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerFactory`.
//!
//! Java's constructor registers ~22 handlers against a live `FantasyFootballClient`.
//! `FantasyFootballClient` is still a GUI-side stub (see
//! `crates/ffb-client/src/client/FantasyFootballClient.rs`), so handlers here are
//! constructed with no client argument (each concrete `ClientCommandHandler*`
//! constructor already documents its own missing-client deviations). Registrations
//! for handler types not yet translated (by the concurrently-running sibling agent
//! covering sketch/session/zap handlers) are left as `// java:` comments so the
//! full Java registration order stays visible; they're added for real once those
//! files land.

use std::collections::HashMap;

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;
use ffb_protocol::net_command::NetCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_add_player::ClientCommandHandlerAddPlayer;
use crate::client::handler::client_command_handler_admin_message::ClientCommandHandlerAdminMessage;
use crate::client::handler::client_command_handler_clear_sketches::ClientCommandHandlerClearSketches;
use crate::client::handler::client_command_handler_game_state::ClientCommandHandlerGameState;
use crate::client::handler::client_command_handler_join::ClientCommandHandlerJoin;
use crate::client::handler::client_command_handler_leave::ClientCommandHandlerLeave;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;
use crate::client::handler::client_command_handler_model_sync::ClientCommandHandlerModelSync;
use crate::client::handler::client_command_handler_remove_player::ClientCommandHandlerRemovePlayer;
use crate::client::handler::client_command_handler_set_prevent_sketching::ClientCommandHandlerSetPreventSketching;
use crate::client::handler::client_command_handler_talk::ClientCommandHandlerTalk;
use crate::client::handler::client_command_handler_user_settings::ClientCommandHandlerUserSettings;

/// Java: `private final Map<NetCommandId, ClientCommandHandler> fCommandHandlerById;`.
pub struct ClientCommandHandlerFactory {
    command_handler_by_id: HashMap<NetCommandId, Box<dyn ClientCommandHandler>>,
}

impl ClientCommandHandlerFactory {
    /// Java: `public ClientCommandHandlerFactory(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        let mut factory = Self { command_handler_by_id: HashMap::new() };

        factory.register(Box::new(ClientCommandHandlerJoin::new()));
        factory.register(Box::new(ClientCommandHandlerLeave::new()));
        factory.register(Box::new(ClientCommandHandlerTalk::new()));
        factory.register(Box::new(ClientCommandHandlerGameState::new()));
        // java: register(new ClientCommandHandlerSound(getClient())); — not yet translated
        factory.register(Box::new(ClientCommandHandlerUserSettings::new()));
        factory.register(Box::new(ClientCommandHandlerAdminMessage::new()));
        factory.register(Box::new(ClientCommandHandlerModelSync::new()));
        // java: register(new ClientCommandHandlerSocketClosed(getClient())); — not yet translated
        factory.register(Box::new(ClientCommandHandlerAddPlayer::new()));
        factory.register(Box::new(ClientCommandHandlerRemovePlayer::new()));
        // java: register(new ClientCommandHandlerGameTime(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerZapPlayer(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerUnzapPlayer(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerUpdateLocalPlayerMarkers(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerAddSketches(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerRemoveSketches(getClient())); — not yet translated
        factory.register(Box::new(ClientCommandHandlerClearSketches::new()));
        // java: register(new ClientCommandHandlerSketchAddCoordinate(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerSketchSetColor(getClient())); — not yet translated
        // java: register(new ClientCommandHandlerSketchSetLabel(getClient())); — not yet translated
        factory.register(Box::new(ClientCommandHandlerSetPreventSketching::new()));

        factory
    }

    /// Java: `private void register(ClientCommandHandler pCommandHandler)`.
    fn register(&mut self, handler: Box<dyn ClientCommandHandler>) {
        self.command_handler_by_id.insert(handler.get_id(), handler);
    }

    /// Java: `public ClientCommandHandler getCommandHandler(NetCommandId pType)`.
    pub fn get_command_handler(&self, id: NetCommandId) -> Option<&dyn ClientCommandHandler> {
        self.command_handler_by_id.get(&id).map(|handler| handler.as_ref())
    }

    /// Java: `public void handleNetCommand(NetCommand pNetCommand, ClientCommandHandlerMode pMode)`.
    ///
    /// Deviation: Java blocks the calling thread on `wait()` when the handler
    /// returns `false` while `pMode == PLAYING`, until another thread calls
    /// `updateClientState(...)` (`notifyAll()`). That cross-thread synchronization
    /// has no meaningful single-threaded Rust translation and is omitted; this
    /// returns the handler's own completion result (or `true`, matching Java's
    /// "no handler found" branch which calls `updateClientState` immediately)
    /// instead of blocking.
    pub fn handle_net_command(&mut self, net_command: &AnyServerCommand, mode: ClientCommandHandlerMode) -> bool {
        match self.command_handler_by_id.get_mut(&net_command.get_id()) {
            Some(handler) => handler.handle_net_command(net_command, mode),
            None => {
                // java: updateClientState(pNetCommand, false);
                true
            }
        }
    }
}

impl Default for ClientCommandHandlerFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::SoundId;
    use ffb_protocol::commands::server_command_admin_message::ServerCommandAdminMessage;
    use ffb_protocol::commands::server_command_sound::ServerCommandSound;

    #[test]
    fn registers_all_currently_translated_handlers() {
        let factory = ClientCommandHandlerFactory::new();
        assert!(factory.get_command_handler(NetCommandId::ServerJoin).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerLeave).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerTalk).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerGameState).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerUserSettings).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerAdminMessage).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerModelSync).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerAddPlayer).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerRemovePlayer).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerClearSketches).is_some());
        assert!(factory.get_command_handler(NetCommandId::ServerSetPreventSketching).is_some());
    }

    #[test]
    fn returns_none_for_a_not_yet_translated_command() {
        let factory = ClientCommandHandlerFactory::new();
        assert!(factory.get_command_handler(NetCommandId::ServerSound).is_none());
    }

    #[test]
    fn handle_net_command_dispatches_to_the_registered_handler() {
        let mut factory = ClientCommandHandlerFactory::new();
        let cmd = AnyServerCommand::ServerAdminMessage(ServerCommandAdminMessage::new(vec!["hi".into()]));
        assert!(factory.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn handle_net_command_returns_true_when_no_handler_is_registered() {
        let mut factory = ClientCommandHandlerFactory::new();
        let cmd = AnyServerCommand::ServerSound(ServerCommandSound::new(SoundId::TOUCHDOWN));
        assert!(factory.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
    }

    #[test]
    fn default_constructs_a_factory_with_handlers_registered() {
        let factory = ClientCommandHandlerFactory::default();
        assert!(factory.get_command_handler(NetCommandId::ServerJoin).is_some());
    }
}
