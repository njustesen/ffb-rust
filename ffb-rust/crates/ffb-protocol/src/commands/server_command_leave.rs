use ffb_model::model::client_mode::ClientMode;
use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandLeave`.
/// Notifies clients when a coach leaves the game.
#[derive(Debug, Clone)]
pub struct ServerCommandLeave {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fCoach` — leaving coach name.
    pub coach: String,
    /// Java: `fClientMode` — connection mode of the leaving coach.
    pub client_mode: ClientMode,
    /// Java: `spectators` — updated list of spectator coach names.
    pub spectators: Vec<String>,
}

impl ServerCommandLeave {
    pub fn new(
        coach: impl Into<String>,
        client_mode: ClientMode,
        spectators: Vec<String>,
    ) -> Self {
        Self { command_nr: 0, coach: coach.into(), client_mode, spectators }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_client_mode(&self) -> ClientMode { self.client_mode }
    pub fn get_spectators(&self) -> &[String] { &self.spectators }
    pub fn get_spectator_count(&self) -> usize { self.spectators.len() }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandLeave.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        map.insert("clientMode".to_string(), serde_json::json!(self.client_mode.get_name()));
        map.insert("spectators".to_string(), serde_json::json!(self.get_spectator_count() as i64));
        map.insert("spectatorNames".to_string(), serde_json::json!(self.spectators));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandLeave.initFrom(source, jsonValue)`. Java falls
    /// back to a bare `spectators` count when `spectatorNames` is absent,
    /// but the Rust struct only tracks the names themselves (the count is
    /// derived), so that legacy fallback path cannot reconstruct the list.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let spectators = json
            .get("spectatorNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(str::to_string).collect())
            .unwrap_or_default();
        Self {
            command_nr: base.command_nr,
            coach: json.get("coach").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            client_mode: json
                .get("clientMode")
                .and_then(|v| v.as_str())
                .and_then(ClientMode::for_name)
                .unwrap_or(ClientMode::PLAYER),
            spectators,
        }
    }
}

impl Default for ServerCommandLeave {
    fn default() -> Self {
        Self {
            command_nr: 0,
            coach: String::new(),
            client_mode: ClientMode::PLAYER,
            spectators: Vec::new(),
        }
    }
}

impl NetCommand for ServerCommandLeave {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerLeave
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
    fn fields_stored() {
        let cmd = ServerCommandLeave::new("Bob", ClientMode::SPECTATOR, vec!["Charlie".into()]);
        assert_eq!(cmd.get_coach(), "Bob");
        assert_eq!(cmd.get_client_mode(), ClientMode::SPECTATOR);
        assert_eq!(cmd.get_spectator_count(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandLeave::default();
        assert!(cmd.coach.is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ServerCommandLeave::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ServerCommandLeave::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandLeave::default().clone();
    }

    #[test]
    fn get_id_is_server_leave() {
        assert_eq!(ServerCommandLeave::default().get_id(), NetCommandId::ServerLeave);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandLeave::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ServerCommandLeave::new("Bob", ClientMode::SPECTATOR, vec!["Charlie".into()]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverLeave");
        assert_eq!(json["coach"], "Bob");
        assert_eq!(json["clientMode"], "spectator");
        assert_eq!(json["spectators"], 1);
    }

    #[test]
    fn round_trip_with_spectators() {
        let mut cmd = ServerCommandLeave::new("Bob", ClientMode::PLAYER, vec!["Charlie".into(), "Dave".into()]);
        cmd.command_nr = 4;
        let json = cmd.to_json_value();
        let restored = ServerCommandLeave::from_json(&json);
        assert_eq!(restored.command_nr, 4);
        assert_eq!(restored.coach, "Bob");
        assert_eq!(restored.client_mode, ClientMode::PLAYER);
        assert_eq!(restored.spectators, vec!["Charlie".to_string(), "Dave".to_string()]);
    }

    #[test]
    fn round_trip_with_defaults() {
        let cmd = ServerCommandLeave::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandLeave::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.spectators.is_empty());
    }
}
