//! 1:1 translation of `com.fumbbl.ffb.client.handler.ClientCommandHandlerSound`.

use ffb_model::enums::NetCommandId;
use ffb_model::model::SoundId;
use ffb_protocol::commands::any_server_command::AnyServerCommand;

use crate::client::handler::client_command_handler::ClientCommandHandler;
use crate::client::handler::client_command_handler_mode::ClientCommandHandlerMode;

#[derive(Debug, Default)]
pub struct ClientCommandHandlerSound {
    /// Not present in Java — records the sound extracted from the most
    /// recently handled command so pure-logic tests can assert on it without
    /// a real `SoundEngine` (see `playSound` note below).
    pub last_sound: Option<SoundId>,
}

impl ClientCommandHandlerSound {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ClientCommandHandler for ClientCommandHandlerSound {
    /// Java: `getId()`.
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSound
    }

    /// Java:
    /// ```java
    /// ServerCommandSound soundCommand = (ServerCommandSound) pNetCommand;
    /// playSound(soundCommand.getSound(), pMode, false);
    /// return true;
    /// ```
    /// `playSound` (base class `ClientCommandHandler.playSound`) reads the
    /// `SETTING_SOUND_MODE` client property and drives `SoundEngine` — both
    /// GUI/property-store concerns with no Rust equivalent yet. The pure part
    /// (extracting which sound to play) is translated; the actual playback
    /// call is left as a `// java:` note.
    fn handle_net_command(
        &mut self,
        net_command: &AnyServerCommand,
        _mode: ClientCommandHandlerMode,
    ) -> bool {
        if let AnyServerCommand::ServerSound(sound_command) = net_command {
            self.last_sound = Some(sound_command.get_sound());
            // java: playSound(soundCommand.getSound(), pMode, false);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_is_server_sound() {
        assert_eq!(ClientCommandHandlerSound::new().get_id(), NetCommandId::ServerSound);
    }

    #[test]
    fn handle_net_command_records_the_sound() {
        let mut handler = ClientCommandHandlerSound::new();
        let cmd = AnyServerCommand::ServerSound(
            ffb_protocol::commands::server_command_sound::ServerCommandSound::new(SoundId::BLOCK),
        );
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(handler.last_sound, Some(SoundId::BLOCK));
    }

    #[test]
    fn handle_net_command_ignores_mismatched_command_type() {
        let mut handler = ClientCommandHandlerSound::new();
        let cmd = AnyServerCommand::ServerClearSketches(
            ffb_protocol::commands::server_command_clear_sketches::ServerCommandClearSketches::new(),
        );
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::PLAYING));
        assert_eq!(handler.last_sound, None);
    }

    #[test]
    fn handle_net_command_returns_true_across_all_modes() {
        let mut handler = ClientCommandHandlerSound::new();
        let cmd = AnyServerCommand::ServerSound(
            ffb_protocol::commands::server_command_sound::ServerCommandSound::new(SoundId::CATCH),
        );
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::REPLAYING));
        assert!(handler.handle_net_command(&cmd, ClientCommandHandlerMode::QUEUING));
    }

    #[test]
    fn default_constructs_a_handler_with_no_sound_yet() {
        let handler = ClientCommandHandlerSound::default();
        assert_eq!(handler.last_sound, None);
    }
}
