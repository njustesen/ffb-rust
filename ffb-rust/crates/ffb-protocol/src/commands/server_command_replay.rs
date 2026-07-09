use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;

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
}

impl Default for ServerCommandReplay {
    fn default() -> Self {
        Self::new()
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
}
