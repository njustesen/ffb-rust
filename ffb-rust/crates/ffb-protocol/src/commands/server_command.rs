use serde::{Deserialize, Serialize};

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommand`.
/// Abstract base for all server-originated commands. Carries a monotonically
/// increasing `command_nr` used for replay sequencing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCommand {
    /// Java: `fCommandNr` — monotonically increasing sequence number for replay.
    pub command_nr: i32,
}

impl ServerCommand {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command_nr(command_nr: i32) -> Self {
        Self { command_nr }
    }

    /// Java: `isReplayable()` — most server commands are replayable by default.
    pub fn is_replayable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_command_nr_is_zero() {
        assert_eq!(ServerCommand::new().command_nr, 0);
    }

    #[test]
    fn with_command_nr_sets_field() {
        let cmd = ServerCommand::with_command_nr(7);
        assert_eq!(cmd.command_nr, 7);
    }

    #[test]
    fn is_replayable_default_true() {
        assert!(ServerCommand::new().is_replayable());
    }

    #[test]
    fn serde_round_trip() {
        let cmd = ServerCommand::with_command_nr(42);
        let json = serde_json::to_string(&cmd).unwrap();
        let back: ServerCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(back.command_nr, 42);
    }
}
