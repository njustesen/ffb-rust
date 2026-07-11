use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandPasswordChallenge`.
/// Sends a password challenge string to the client for authentication.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandPasswordChallenge {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fChallenge` — the challenge string.
    pub challenge: String,
}

impl ServerCommandPasswordChallenge {
    pub fn new(challenge: impl Into<String>) -> Self {
        Self { command_nr: 0, challenge: challenge.into() }
    }
    pub fn get_challenge(&self) -> &str { &self.challenge }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandPasswordChallenge.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("challenge".to_string(), serde_json::json!(self.challenge));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandPasswordChallenge.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            challenge: json.get("challenge").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl NetCommand for ServerCommandPasswordChallenge {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerPasswordChallenge
    }

    /// Java: `getContext()` override — returns `FactoryContext.APPLICATION`.
    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_stored() {
        let cmd = ServerCommandPasswordChallenge::new("abc123");
        assert_eq!(cmd.get_challenge(), "abc123");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandPasswordChallenge::default();
        assert!(cmd.challenge.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandPasswordChallenge::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandPasswordChallenge::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandPasswordChallenge::default());
        assert!(s.contains("ServerCommandPasswordChallenge"));
    }

    #[test]
    fn get_id_is_server_password_challenge() {
        assert_eq!(ServerCommandPasswordChallenge::default().get_id(), NetCommandId::ServerPasswordChallenge);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandPasswordChallenge::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_challenge() {
        let cmd = ServerCommandPasswordChallenge::new("abc123");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverPasswordChallenge");
        assert_eq!(json["challenge"], "abc123");
    }

    #[test]
    fn round_trip_with_challenge() {
        let mut cmd = ServerCommandPasswordChallenge::new("xyz789");
        cmd.command_nr = 2;
        let json = cmd.to_json_value();
        let restored = ServerCommandPasswordChallenge::from_json(&json);
        assert_eq!(restored.command_nr, 2);
        assert_eq!(restored.challenge, "xyz789");
    }

    #[test]
    fn round_trip_with_no_challenge() {
        let cmd = ServerCommandPasswordChallenge::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandPasswordChallenge::from_json(&json);
        assert!(restored.challenge.is_empty());
    }
}
