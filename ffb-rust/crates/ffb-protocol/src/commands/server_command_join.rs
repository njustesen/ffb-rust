use ffb_model::model::client_mode::ClientMode;
use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandJoin`.
/// Notifies clients when a coach joins the game.
#[derive(Debug, Clone)]
pub struct ServerCommandJoin {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fCoach` — coach name.
    pub coach: String,
    /// Java: `fClientMode` — connection mode (Home/Away/Spectator/Replay).
    pub client_mode: ClientMode,
    /// Java: `fPlayerNames` — list of logged-in player names.
    pub player_names: Vec<String>,
    /// Java: `spectators` — list of spectator coach names.
    pub spectators: Vec<String>,
    /// Java: `replayName` — name of replay being watched (if any).
    pub replay_name: String,
}

impl ServerCommandJoin {
    pub fn new(
        coach: impl Into<String>,
        client_mode: ClientMode,
        player_names: Vec<String>,
        spectators: Vec<String>,
        replay_name: impl Into<String>,
    ) -> Self {
        Self {
            command_nr: 0,
            coach: coach.into(),
            client_mode,
            player_names,
            spectators,
            replay_name: replay_name.into(),
        }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_client_mode(&self) -> ClientMode { self.client_mode }
    pub fn get_player_names(&self) -> &[String] { &self.player_names }
    pub fn get_spectators(&self) -> &[String] { &self.spectators }
    pub fn get_spectator_count(&self) -> usize { self.spectators.len() }
    pub fn get_replay_name(&self) -> &str { &self.replay_name }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandJoin.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("coach".to_string(), serde_json::json!(self.coach));
        map.insert("clientMode".to_string(), serde_json::json!(self.client_mode.get_name()));
        map.insert("spectators".to_string(), serde_json::json!(self.get_spectator_count() as i64));
        map.insert("playerNames".to_string(), serde_json::json!(self.player_names));
        map.insert("spectatorNames".to_string(), serde_json::json!(self.spectators));
        map.insert("name".to_string(), serde_json::json!(self.replay_name));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandJoin.initFrom(source, jsonValue)`. Java falls
    /// back to a bare `spectators` count when `spectatorNames` is absent,
    /// but the Rust struct only tracks the names themselves (the count is
    /// derived), so that legacy fallback path cannot reconstruct the list.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let player_names = json
            .get("playerNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(str::to_string).collect())
            .unwrap_or_default();
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
            player_names,
            spectators,
            replay_name: json.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        }
    }
}

impl Default for ServerCommandJoin {
    fn default() -> Self {
        Self {
            command_nr: 0,
            coach: String::new(),
            client_mode: ClientMode::PLAYER,
            player_names: Vec::new(),
            spectators: Vec::new(),
            replay_name: String::new(),
        }
    }
}

impl NetCommand for ServerCommandJoin {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerJoin
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
        let cmd = ServerCommandJoin::new(
            "Alice",
            ClientMode::PLAYER,
            vec!["Alice".into()],
            vec!["Bob".into()],
            "",
        );
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_client_mode(), ClientMode::PLAYER);
        assert_eq!(cmd.get_spectator_count(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandJoin::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.player_names.is_empty());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ServerCommandJoin::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ServerCommandJoin::default();
        let _ = cmd.clone();
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandJoin::default().clone();
    }

    #[test]
    fn get_id_is_server_join() {
        assert_eq!(ServerCommandJoin::default().get_id(), NetCommandId::ServerJoin);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandJoin::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_coach() {
        let cmd = ServerCommandJoin::new("Alice", ClientMode::SPECTATOR, vec![], vec!["Bob".into()], "");
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverJoin");
        assert_eq!(json["coach"], "Alice");
        assert_eq!(json["clientMode"], "spectator");
        assert_eq!(json["spectators"], 1);
    }

    #[test]
    fn round_trip_with_players_and_spectators() {
        let mut cmd = ServerCommandJoin::new(
            "Alice",
            ClientMode::PLAYER,
            vec!["Alice".into(), "Bob".into()],
            vec!["Carol".into()],
            "replay1",
        );
        cmd.command_nr = 6;
        let json = cmd.to_json_value();
        let restored = ServerCommandJoin::from_json(&json);
        assert_eq!(restored.command_nr, 6);
        assert_eq!(restored.coach, "Alice");
        assert_eq!(restored.client_mode, ClientMode::PLAYER);
        assert_eq!(restored.player_names, vec!["Alice".to_string(), "Bob".to_string()]);
        assert_eq!(restored.spectators, vec!["Carol".to_string()]);
        assert_eq!(restored.replay_name, "replay1");
    }

    #[test]
    fn round_trip_with_defaults() {
        let cmd = ServerCommandJoin::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandJoin::from_json(&json);
        assert!(restored.coach.is_empty());
        assert!(restored.player_names.is_empty());
        assert!(restored.spectators.is_empty());
    }
}
