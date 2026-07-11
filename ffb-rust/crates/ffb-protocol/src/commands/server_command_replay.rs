use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandReplay`.
/// Carries a batch of replay commands plus metadata about the total replay size.
/// Java: `isReplayable()` returns `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommandReplay {
    /// Java: `fCommandNr` inherited from `ServerCommand`.
    pub command_nr: i32,
    /// Java: `fTotalNrOfCommands`.
    pub total_nr_of_commands: i32,
    /// Java: `lastCommand`.
    pub last_command: bool,
    /// The batch of server commands in this replay chunk.
    pub replay_commands: Vec<ServerCommand>,
    /// Indices of commands that affect player markings.
    pub marking_affecting_commands: HashSet<i32>,
}

impl ServerCommandReplay {
    /// Java: `MAX_NR_OF_COMMANDS = 100`.
    pub const MAX_NR_OF_COMMANDS: usize = 100;

    pub const ID: NetCommandId = NetCommandId::ServerReplay;

    pub fn new() -> Self {
        Self {
            command_nr: 0,
            total_nr_of_commands: 0,
            last_command: false,
            replay_commands: Vec::new(),
            marking_affecting_commands: HashSet::new(),
        }
    }

    /// Append a single command to the batch.
    pub fn add(&mut self, cmd: ServerCommand) {
        self.replay_commands.push(cmd);
    }

    pub fn nr_of_commands(&self) -> usize {
        self.replay_commands.len()
    }

    pub fn find_highest_command_nr(&self) -> i32 {
        self.replay_commands.iter().map(|c| c.command_nr).max().unwrap_or(0)
    }

    pub fn find_lowest_command_nr(&self) -> i32 {
        self.replay_commands.iter().map(|c| c.command_nr).min().unwrap_or(i32::MAX)
    }

    pub fn add_marking_affecting_command(&mut self, index: i32) {
        self.marking_affecting_commands.insert(index);
    }

    /// Java: `isReplayable()` — replay bundles are not themselves replayed.
    pub fn is_replayable(&self) -> bool {
        false
    }

    pub fn id(&self) -> NetCommandId {
        Self::ID
    }

    /// Java: `ServerCommandReplay.toJsonValue()`. `fReplayCommands` holds
    /// polymorphic `ServerCommand` subclasses in Java (each serialized via
    /// its own `toJsonValue()`); the Rust struct only stores the shared
    /// `ServerCommand` base shape (`command_nr`), so each array element is
    /// serialized as just its `commandNr` field — a deliberate simplification
    /// matching what data the struct actually holds.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("totalNrOfCommands".to_string(), serde_json::json!(self.total_nr_of_commands));
        let command_array: Vec<serde_json::Value> = self
            .replay_commands
            .iter()
            .map(|c| serde_json::json!({ "commandNr": c.command_nr }))
            .collect();
        map.insert("commandArray".to_string(), serde_json::Value::Array(command_array));
        map.insert("lastCommand".to_string(), serde_json::json!(self.last_command));
        let mut markings: Vec<i32> = self.marking_affecting_commands.iter().copied().collect();
        markings.sort_unstable();
        map.insert("markingIntervalIndexes".to_string(), serde_json::json!(markings));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandReplay.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let replay_commands = json
            .get("commandArray")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .map(|v| ServerCommand {
                        command_nr: v.get("commandNr").and_then(|n| n.as_i64()).unwrap_or(0) as i32,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let marking_affecting_commands = json
            .get("markingIntervalIndexes")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect())
            .unwrap_or_default();
        Self {
            command_nr: base.command_nr,
            total_nr_of_commands: json.get("totalNrOfCommands").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            last_command: json.get("lastCommand").and_then(|v| v.as_bool()).unwrap_or(false),
            replay_commands,
            marking_affecting_commands,
        }
    }
}

impl Default for ServerCommandReplay {
    fn default() -> Self {
        Self::new()
    }
}

impl NetCommand for ServerCommandReplay {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerReplay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let r = ServerCommandReplay::new();
        assert_eq!(r.nr_of_commands(), 0);
        assert!(!r.last_command);
    }

    #[test]
    fn add_increments_count() {
        let mut r = ServerCommandReplay::new();
        r.add(ServerCommand::with_command_nr(1));
        r.add(ServerCommand::with_command_nr(2));
        assert_eq!(r.nr_of_commands(), 2);
    }

    #[test]
    fn find_highest_and_lowest() {
        let mut r = ServerCommandReplay::new();
        r.add(ServerCommand::with_command_nr(3));
        r.add(ServerCommand::with_command_nr(7));
        r.add(ServerCommand::with_command_nr(1));
        assert_eq!(r.find_highest_command_nr(), 7);
        assert_eq!(r.find_lowest_command_nr(), 1);
    }

    #[test]
    fn not_replayable() {
        assert!(!ServerCommandReplay::new().is_replayable());
    }

    #[test]
    fn id_is_server_replay() {
        assert_eq!(ServerCommandReplay::new().id(), NetCommandId::ServerReplay);
    }

    #[test]
    fn max_nr_of_commands_constant() {
        assert_eq!(ServerCommandReplay::MAX_NR_OF_COMMANDS, 100);
    }

    #[test]
    fn get_id_is_server_replay() {
        assert_eq!(ServerCommandReplay::new().get_id(), NetCommandId::ServerReplay);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_command_array() {
        let mut r = ServerCommandReplay::new();
        r.command_nr = 4;
        r.total_nr_of_commands = 50;
        r.last_command = true;
        r.add(ServerCommand::with_command_nr(1));
        r.add_marking_affecting_command(2);
        let json = r.to_json_value();
        assert_eq!(json["netCommandId"], "serverReplay");
        assert_eq!(json["commandNr"], 4);
        assert_eq!(json["totalNrOfCommands"], 50);
        assert_eq!(json["lastCommand"], true);
        assert_eq!(json["commandArray"][0]["commandNr"], 1);
        assert_eq!(json["markingIntervalIndexes"][0], 2);
    }

    #[test]
    fn round_trip_with_data() {
        let mut r = ServerCommandReplay::new();
        r.command_nr = 3;
        r.total_nr_of_commands = 10;
        r.last_command = true;
        r.add(ServerCommand::with_command_nr(5));
        r.add(ServerCommand::with_command_nr(6));
        r.add_marking_affecting_command(1);
        let json = r.to_json_value();
        let restored = ServerCommandReplay::from_json(&json);
        assert_eq!(restored.command_nr, 3);
        assert_eq!(restored.total_nr_of_commands, 10);
        assert!(restored.last_command);
        assert_eq!(restored.nr_of_commands(), 2);
        assert_eq!(restored.replay_commands[0].command_nr, 5);
        assert!(restored.marking_affecting_commands.contains(&1));
    }

    #[test]
    fn round_trip_with_default_empty() {
        let r = ServerCommandReplay::new();
        let json = r.to_json_value();
        let restored = ServerCommandReplay::from_json(&json);
        assert_eq!(restored.nr_of_commands(), 0);
        assert!(!restored.last_command);
        assert!(restored.marking_affecting_commands.is_empty());
    }
}
