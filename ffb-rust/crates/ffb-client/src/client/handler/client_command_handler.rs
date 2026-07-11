//! Minimal 1:1 shape of `com.fumbbl.ffb.client.handler.ClientCommandHandler`
//! (the abstract base class every `ClientCommandHandler*` subclass extends).
//!
//! The Java class also carries a `FantasyFootballClient` reference and
//! `playSound`/`refreshFieldComponent`/`refreshSideBars`/`refreshGameMenuBar`/
//! `updateDialog`/`updateGameTitle` helpers, all of which are thin wrappers
//! around Swing/AWT `UserInterface` calls. `ClientData.rs` and
//! `FantasyFootballClient.rs` are still stubs with no real fields (out of
//! scope here — see TRANSLATION_TRACKER.md), so those GUI helpers have no
//! Rust equivalent yet; each concrete handler notes the Java call it would
//! make with a `// java:` comment instead of inventing a client API.
//!
//! This trait captures only the two `abstract` methods every subclass
//! overrides: `getId()` and `handleNetCommand(NetCommand, ClientCommandHandlerMode)`.

use ffb_model::enums::NetCommandId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

pub trait ClientCommandHandler {
    /// Java: `ClientCommandHandler.getId()`.
    fn get_id(&self) -> NetCommandId;

    /// Java: `ClientCommandHandler.handleNetCommand(NetCommand, ClientCommandHandlerMode)`.
    fn handle_net_command(
        &mut self,
        net_command: &AnyServerCommand,
        mode: ClientCommandHandlerMode,
    ) -> bool;
}
