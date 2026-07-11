/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSound.
/// Handles `/sound` command — plays a named sound to all game clients.
use ffb_model::factory::sound_id_factory::SoundIdFactory;
use ffb_model::model::SoundId;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerSound {
    pub required_client: Client,
    pub required_environment: Environment,
    sound_id_factory: SoundIdFactory,
}

impl TalkHandlerSound {
    pub const COMMAND: &'static str = "/sound";
    pub const COMMAND_PARTS_THRESHOLD: usize = 1;

    /// Java: `super("/sound", 1, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        Self {
            required_client: Client::Player,
            required_environment: Environment::TestGame,
            sound_id_factory: SoundIdFactory::default(),
        }
    }

    /// Java: `handle(...)` — looks up `commands[1]` via `SoundIdFactory.forName` and, when
    /// found, returns the confirmation message plus the resolved `SoundId` to broadcast
    /// (Java: `sendPlayerTalk` + `sendSound`).
    pub fn handle(&self, commands: &[String]) -> Option<(String, SoundId)> {
        let name = commands.get(1)?;
        let sound_id = self.sound_id_factory.for_name(name)?;
        Some((format!("Playing sound {}", sound_id.get_name()), sound_id))
    }
}

impl Default for TalkHandlerSound {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let h = TalkHandlerSound::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }

    #[test]
    fn handle_resolves_known_sound() {
        let h = TalkHandlerSound::new();
        let commands = vec!["/sound".to_string(), "block".to_string()];
        let (info, sound) = h.handle(&commands).unwrap();
        assert_eq!(sound, SoundId::BLOCK);
        assert!(info.contains("block"));
    }

    #[test]
    fn handle_unknown_sound_returns_none() {
        let h = TalkHandlerSound::new();
        let commands = vec!["/sound".to_string(), "not_a_sound".to_string()];
        assert!(h.handle(&commands).is_none());
    }

    #[test]
    fn handle_missing_argument_returns_none() {
        let h = TalkHandlerSound::new();
        let commands = vec!["/sound".to_string()];
        assert!(h.handle(&commands).is_none());
    }
}
