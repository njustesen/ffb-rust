//! 1:1 translation of `com.fumbbl.ffb.client.handler.AbstractClientCommandHandlerSketch<S>`.
//!
//! Java models this as a generic abstract class parameterized by the concrete
//! `ServerCommand` subtype `S`; subclasses only override `updateSketchManager(S)`
//! while `handleNetCommand` is `final` on the base class. Rust has no matching
//! generic-over-command-subtype hierarchy (commands are variants of the single
//! `AnyServerCommand` sum type instead), so this is translated as a trait with
//! a provided (`final`-equivalent) `handle_net_command` default method that
//! calls the overridden `update_sketch_manager` hook.

use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

pub trait AbstractClientCommandHandlerSketch {
    /// Java: abstract `updateSketchManager(S command)`.
    fn update_sketch_manager(&mut self, command: &AnyServerCommand);

    /// Java: `final boolean handleNetCommand(NetCommand command, ClientCommandHandlerMode pMode)`.
    ///
    /// ```java
    /// updateSketchManager((S) command);
    /// ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
    /// SketchState sketchState = new SketchState(sketchManager.getAllSketches());
    /// getClient().getGame().notifyObservers(new ModelChange(ModelChangeId.SKETCH_UPDATE, null, sketchState));
    /// return true;
    /// ```
    /// `ClientSketchManager` (`client/overlay/sketch/ClientSketchManager.rs`) is still a
    /// GUI-coupled stub with no fields, and `Game.notifyObservers` has no Rust equivalent
    /// (the observer/model-change pattern is out of scope here) — both are left as
    /// `// java:` notes rather than invented APIs.
    fn handle_net_command(
        &mut self,
        command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        self.update_sketch_manager(command);
        // java: ClientSketchManager sketchManager = getClient().getUserInterface().getSketchManager();
        // java: SketchState sketchState = new SketchState(sketchManager.getAllSketches());
        // java: getClient().getGame().notifyObservers(new ModelChange(ModelChangeId.SKETCH_UPDATE, null, sketchState));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches;

    struct Recorder {
        calls: u32,
    }

    impl AbstractClientCommandHandlerSketch for Recorder {
        fn update_sketch_manager(&mut self, _command: &AnyServerCommand) {
            self.calls += 1;
        }
    }

    #[test]
    fn handle_net_command_calls_update_sketch_manager() {
        let mut r = Recorder { calls: 0 };
        let cmd = AnyServerCommand::ServerClearSketches(ServerCommandClearSketches::new());
        let handled = r.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING);
        assert_eq!(r.calls, 1);
        assert!(handled);
    }

    #[test]
    fn handle_net_command_always_returns_true() {
        let mut r = Recorder { calls: 0 };
        let cmd = AnyServerCommand::ServerClearSketches(ServerCommandClearSketches::new());
        assert!(r.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(r.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }
}
