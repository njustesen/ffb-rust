/// 1:1 translation of com.fumbbl.ffb.server.handler.AbstractServerCommandHandlerSketch.
///
/// Java is a generic abstract class `<C extends ClientSketchCommand, S extends ServerCommand>`
/// extending `ServerCommandHandler`, with two abstract methods (`updateSketchManager`,
/// `createServerCommand`) implemented by each concrete `ServerCommandHandlerXxxSketch`
/// subclass, plus one `final` method (`handleCommand`) shared by all of them.
///
/// Rust has no class inheritance, so the shared `handleCommand` logic lives here as a generic
/// struct/method parameterized by a `SketchCommandOps` trait that plays the role of the two
/// abstract methods; concrete handlers implement that trait and are handed to this struct by
/// composition instead of extending it.
use std::sync::{Arc, Mutex};

use ffb_engine::server_sketch_manager::ServerSketchManager;

use crate::model::received_command::SessionId;
use crate::net::replay_session_manager::ReplaySessionManager;

/// Java: the two abstract methods `updateSketchManager(Session, C)` and
/// `createServerCommand(String, C)`, plus `C.requiresControl()` (declared on
/// `ClientSketchCommand`, but overridable per concrete command type).
pub trait SketchCommandOps {
    /// Concrete client sketch command type (Java generic `C`).
    type Command;
    /// Concrete server command type sent back out (Java generic `S`).
    type ServerCmd;

    /// Java: `C.requiresControl()`. `ClientSketchCommand.requiresControl()` always returns
    /// `false`; the default mirrors that.
    fn requires_control(&self, _command: &Self::Command) -> bool {
        false
    }

    /// Java: `updateSketchManager(Session, C)`.
    fn update_sketch_manager(
        &self,
        sketch_manager: &mut ServerSketchManager,
        session: SessionId,
        command: &Self::Command,
    );

    /// Java: `createServerCommand(String, C)`.
    fn create_server_command(&self, coach: &str, command: &Self::Command) -> Self::ServerCmd;
}

/// Java: `AbstractServerCommandHandlerSketch<C, S>`.
pub struct AbstractServerCommandHandlerSketch<O: SketchCommandOps> {
    /// Java: `sketchManager` (from `getServer().getSketchManager()`).
    pub sketch_manager: Arc<Mutex<ServerSketchManager>>,
    /// Java: `replaySessionManager` (from `getServer().getReplaySessionManager()`).
    pub replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
    /// The concrete subclass' implementation of the two abstract methods.
    pub ops: O,
}

impl<O: SketchCommandOps> AbstractServerCommandHandlerSketch<O> {
    /// Java: `protected AbstractServerCommandHandlerSketch(FantasyFootballServer pServer)`.
    pub fn new(
        sketch_manager: Arc<Mutex<ServerSketchManager>>,
        replay_session_manager: Arc<Mutex<ReplaySessionManager>>,
        ops: O,
    ) -> Self {
        Self {
            sketch_manager,
            replay_session_manager,
            ops,
        }
    }

    /// Java: `public final boolean handleCommand(ReceivedCommand receivedCommand)`.
    ///
    /// Returns `(true, sends)` where `sends` is the list of `(session, command)` pairs that
    /// Java would deliver via `getServer().getCommunication().sendToReplaySession(...)`. No
    /// per-session sender is wired for replay sessions in `ReplaySessionManager` yet
    /// (Phase ZV), so the actual network write is left to the caller of this method.
    pub fn handle_command(
        &self,
        session: SessionId,
        command: O::Command,
    ) -> (bool, Vec<(SessionId, O::ServerCmd)>) {
        let mut sends = Vec::new();
        let rsm = self.replay_session_manager.lock().unwrap();

        if rsm.has(session) {
            if self.ops.requires_control(&command) && !rsm.has_control(session) {
                return (true, sends);
            }

            {
                let mut sketch_manager = self.sketch_manager.lock().unwrap();
                self.ops
                    .update_sketch_manager(&mut sketch_manager, session, &command);
            }

            let coach = rsm.coach(session).unwrap_or_default();
            for other in rsm.other_sessions(session) {
                let server_command = self.ops.create_server_command(&coach, &command);
                sends.push((other, server_command));
            }
        }

        (true, sends)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_engine::server_sketch_manager::Sketch;
    use ffb_protocol::commands::client_sketch_command::ClientSketchCommand;

    struct DummyOps;
    impl SketchCommandOps for DummyOps {
        type Command = ClientSketchCommand;
        type ServerCmd = String;

        fn update_sketch_manager(
            &self,
            sketch_manager: &mut ServerSketchManager,
            session: SessionId,
            _command: &ClientSketchCommand,
        ) {
            sketch_manager.add_sketch(&session.to_string(), Sketch::new("s1"));
        }

        fn create_server_command(&self, coach: &str, _command: &ClientSketchCommand) -> String {
            format!("sketch-from-{coach}")
        }
    }

    struct ControlRequiredOps;
    impl SketchCommandOps for ControlRequiredOps {
        type Command = ClientSketchCommand;
        type ServerCmd = String;

        fn requires_control(&self, _command: &ClientSketchCommand) -> bool {
            true
        }

        fn update_sketch_manager(
            &self,
            _sketch_manager: &mut ServerSketchManager,
            _session: SessionId,
            _command: &ClientSketchCommand,
        ) {
        }

        fn create_server_command(&self, _coach: &str, _command: &ClientSketchCommand) -> String {
            String::new()
        }
    }

    #[test]
    fn propagates_to_other_replay_sessions() {
        let sketch_manager = Arc::new(Mutex::new(ServerSketchManager::new()));
        let mut rsm = ReplaySessionManager::new();
        rsm.add_session(1, "r".into(), "coach1".into());
        rsm.add_session(2, "r".into(), "coach2".into());
        let replay_session_manager = Arc::new(Mutex::new(rsm));

        let handler = AbstractServerCommandHandlerSketch::new(
            Arc::clone(&sketch_manager),
            replay_session_manager,
            DummyOps,
        );
        let (ok, sends) = handler.handle_command(1, ClientSketchCommand::new());

        assert!(ok);
        assert_eq!(sends, vec![(2, "sketch-from-coach1".to_string())]);
        assert_eq!(sketch_manager.lock().unwrap().get_sketches("1").len(), 1);
    }

    #[test]
    fn non_replay_session_produces_no_sends() {
        let sketch_manager = Arc::new(Mutex::new(ServerSketchManager::new()));
        let replay_session_manager = Arc::new(Mutex::new(ReplaySessionManager::new()));

        let handler =
            AbstractServerCommandHandlerSketch::new(sketch_manager, replay_session_manager, DummyOps);
        let (ok, sends) = handler.handle_command(99, ClientSketchCommand::new());

        assert!(ok);
        assert!(sends.is_empty());
    }

    #[test]
    fn requires_control_blocks_non_controlling_session() {
        let mut rsm = ReplaySessionManager::new();
        rsm.add_session(1, "r".into(), "coach1".into()); // first session gets control
        rsm.add_session(2, "r".into(), "coach2".into()); // no control
        let replay_session_manager = Arc::new(Mutex::new(rsm));
        let sketch_manager = Arc::new(Mutex::new(ServerSketchManager::new()));

        let handler = AbstractServerCommandHandlerSketch::new(
            sketch_manager,
            replay_session_manager,
            ControlRequiredOps,
        );
        let (ok, sends) = handler.handle_command(2, ClientSketchCommand::new());

        assert!(ok);
        assert!(sends.is_empty());
    }
}
