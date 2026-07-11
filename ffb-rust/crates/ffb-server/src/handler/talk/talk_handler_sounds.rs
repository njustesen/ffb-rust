/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSounds.
/// Handles `/sounds` command — lists all available sound IDs sorted by name.
use super::talk_requirements::{Client, Environment};
use ffb_model::model::sound_id::SoundId;

pub struct TalkHandlerSounds {
    pub required_client: Client,
    pub required_environment: Environment,
}

impl TalkHandlerSounds {
    pub const COMMAND: &'static str = "/sounds";
    pub const COMMAND_PARTS_THRESHOLD: usize = 0;

    /// Java: `super("/sounds", 0, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        Self { required_client: Client::Player, required_environment: Environment::TestGame }
    }

    /// Java: `handle(...)` — collects `SoundId.values()`, sorts by display name, and sends
    /// `["Available sounds:", ...names]` via `sendTalk`.
    pub fn handle(&self) -> Vec<String> {
        Self::sound_listing(&Self::all_sound_names())
    }

    /// Java: `SoundId.values()` mapped to `getName()`.
    fn all_sound_names() -> Vec<String> {
        SoundId::all().iter().map(|s| s.get_name().to_string()).collect()
    }

    /// Java: `String[] info = { "Available sounds:", ...soundNames }` — pure formatting,
    /// testable independently of the (currently blocked) enumeration step above.
    pub fn sound_listing(names: &[String]) -> Vec<String> {
        let mut sorted = names.to_vec();
        sorted.sort();
        let mut info = vec!["Available sounds:".to_string()];
        info.extend(sorted);
        info
    }
}

impl Default for TalkHandlerSounds {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let h = TalkHandlerSounds::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }

    #[test]
    fn sound_listing_prefixes_header_and_sorts() {
        let names = vec!["zap".to_string(), "block".to_string()];
        let listing = TalkHandlerSounds::sound_listing(&names);
        assert_eq!(listing, vec!["Available sounds:", "block", "zap"]);
    }

    #[test]
    fn handle_lists_all_sounds_sorted() {
        let h = TalkHandlerSounds::new();
        let result = h.handle();
        assert_eq!(result[0], "Available sounds:");
        assert_eq!(result.len(), 1 + SoundId::all().len());
        let mut sorted_names: Vec<String> =
            SoundId::all().iter().map(|s| s.get_name().to_string()).collect();
        sorted_names.sort();
        assert_eq!(&result[1..], sorted_names.as_slice());
    }
}
