use ffb_model::model::SoundId;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandSound`.
/// Instructs the client to play a specific sound effect.
#[derive(Debug, Clone)]
pub struct ServerCommandSound {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fSound` — the sound to play.
    pub sound: SoundId,
}

impl ServerCommandSound {
    pub fn new(sound: SoundId) -> Self { Self { command_nr: 0, sound } }
    pub fn get_sound(&self) -> SoundId { self.sound }

    /// Java: `ServerCommandSound.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("sound".to_string(), serde_json::json!(self.sound.get_name()));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandSound.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            sound: json.get("sound").and_then(|v| v.as_str()).and_then(SoundId::for_name).unwrap_or(SoundId::TOUCHDOWN),
        }
    }
}

impl NetCommand for ServerCommandSound {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerSound
    }
}

impl Default for ServerCommandSound {
    fn default() -> Self { Self { command_nr: 0, sound: SoundId::TOUCHDOWN } }
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


    #[test]
    fn clone_roundtrip() {
        let cmd = ServerCommandSound::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandSound::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandSound::default());
        assert!(s.contains("ServerCommandSound"));
    }

    #[test]
    fn get_id_is_server_sound() {
        assert_eq!(ServerCommandSound::default().get_id(), NetCommandId::ServerSound);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_sound() {
        let cmd = ServerCommandSound::new(SoundId::BLOCK);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverSound");
        assert_eq!(json["sound"], "block");
    }

    #[test]
    fn round_trip_with_sound() {
        let mut cmd = ServerCommandSound::new(SoundId::CATCH);
        cmd.command_nr = 3;
        let json = cmd.to_json_value();
        let restored = ServerCommandSound::from_json(&json);
        assert_eq!(restored.command_nr, 3);
        assert_eq!(restored.sound, SoundId::CATCH);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ServerCommandSound::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandSound::from_json(&json);
        assert_eq!(restored.sound, SoundId::TOUCHDOWN);
        assert_eq!(restored.command_nr, 0);
    }
}
