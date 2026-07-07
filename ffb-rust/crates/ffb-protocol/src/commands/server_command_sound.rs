use ffb_model::model::SoundId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSound`.
/// Instructs the client to play a specific sound effect.
#[derive(Debug, Clone)]
pub struct ServerCommandSound {
    /// Java: `fSound` — the sound to play.
    pub sound: SoundId,
}

impl ServerCommandSound {
    pub fn new(sound: SoundId) -> Self { Self { sound } }
    pub fn get_sound(&self) -> SoundId { self.sound }
}

impl Default for ServerCommandSound {
    fn default() -> Self { Self { sound: SoundId::TOUCHDOWN } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_stored() {
        let cmd = ServerCommandSound::new(SoundId::TOUCHDOWN);
        assert_eq!(cmd.get_sound(), SoundId::TOUCHDOWN);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandSound::default()).is_empty());
    }

}
