use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandGameTime`.
/// Sends current game clock and turn clock to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandGameTime {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
    /// Java: `fGameTime` — total elapsed game time in ms.
    pub game_time: i64,
    /// Java: `fTurnTime` — elapsed time for the current turn in ms.
    pub turn_time: i64,
}

impl ServerCommandGameTime {
    pub fn new(game_time: i64, turn_time: i64) -> Self {
        Self { command_nr: 0, game_time, turn_time }
    }
    pub fn get_game_time(&self) -> i64 { self.game_time }
    pub fn get_turn_time(&self) -> i64 { self.turn_time }

    /// Java: `isReplayable()`.
    pub fn is_replayable(&self) -> bool {
        false
    }

    /// Java: `ServerCommandGameTime.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("gameTime".to_string(), serde_json::json!(self.game_time));
        map.insert("turnTime".to_string(), serde_json::json!(self.turn_time));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandGameTime.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        Self {
            command_nr: base.command_nr,
            game_time: json.get("gameTime").and_then(|v| v.as_i64()).unwrap_or(0),
            turn_time: json.get("turnTime").and_then(|v| v.as_i64()).unwrap_or(0),
        }
    }
}

impl NetCommand for ServerCommandGameTime {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerGameTime
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
        let cmd = ServerCommandGameTime::new(60_000, 30_000);
        assert_eq!(cmd.get_game_time(), 60_000);
        assert_eq!(cmd.get_turn_time(), 30_000);
    }

    #[test]
    fn default_zeros() {
        let cmd = ServerCommandGameTime::default();
        assert_eq!(cmd.game_time, 0);
        assert_eq!(cmd.turn_time, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandGameTime::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandGameTime::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandGameTime::default());
        assert!(s.contains("ServerCommandGameTime"));
    }

    #[test]
    fn get_id_is_server_game_time() {
        assert_eq!(ServerCommandGameTime::default().get_id(), NetCommandId::ServerGameTime);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandGameTime::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_times() {
        let mut cmd = ServerCommandGameTime::new(60_000, 30_000);
        cmd.command_nr = 1;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverGameTime");
        assert_eq!(json["commandNr"], 1);
        assert_eq!(json["gameTime"], 60_000);
        assert_eq!(json["turnTime"], 30_000);
    }

    #[test]
    fn round_trip_with_times() {
        let mut cmd = ServerCommandGameTime::new(5000, 2000);
        cmd.command_nr = 8;
        let json = cmd.to_json_value();
        let restored = ServerCommandGameTime::from_json(&json);
        assert_eq!(restored.command_nr, 8);
        assert_eq!(restored.game_time, 5000);
        assert_eq!(restored.turn_time, 2000);
    }

    #[test]
    fn round_trip_with_zero_times() {
        let cmd = ServerCommandGameTime::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandGameTime::from_json(&json);
        assert_eq!(restored.game_time, 0);
        assert_eq!(restored.turn_time, 0);
    }
}
