use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::i_server_log_level::IServerLogLevel;

/// Debug logging utility for the FFB server — 1:1 translation of Java DebugLog.
pub struct DebugLog {
    log_level: i32,
    log_file: PathBuf,
    base_log_path: PathBuf,
    force_log: HashSet<i64>,
    log_files: HashMap<i64, PathBuf>,
    split_logs: bool,
}

impl DebugLog {
    pub const COMMAND_CLIENT_HOME: &'static str = " H->";
    pub const COMMAND_SERVER_HOME: &'static str = " ->H";
    pub const COMMAND_SERVER_HOME_SPECTATORS: &'static str = "->HS";
    pub const COMMAND_CLIENT_AWAY: &'static str = " A->";
    pub const COMMAND_SERVER_AWAY: &'static str = " ->A";
    pub const COMMAND_CLIENT_SPECTATOR: &'static str = " S->";
    pub const COMMAND_CLIENT_REPLAY: &'static str = " R->";
    pub const COMMAND_SERVER_SPECTATOR: &'static str = " ->S";
    pub const COMMAND_CLIENT_UNKNOWN: &'static str = " ?->";
    pub const COMMAND_SERVER_UNKNOWN: &'static str = " ->?";
    pub const COMMAND_SERVER_ALL_CLIENTS: &'static str = "->AC";
    pub const COMMAND_NO_COMMAND: &'static str = "----";
    pub const FUMBBL_REQUEST: &'static str = " ->F";
    pub const FUMBBL_RESPONSE: &'static str = " F->";
    pub const GAME_LOG_SUFFIX: &'static str = ".log";
    pub const GZ_SUFFIX: &'static str = ".gz";

    pub fn new(log_file: PathBuf, base_log_path: PathBuf, log_level: i32, split_logs: bool) -> Self {
        Self {
            log_level,
            log_file,
            base_log_path,
            force_log: HashSet::new(),
            log_files: HashMap::new(),
            split_logs,
        }
    }

    pub fn get_log_level(&self) -> i32 {
        self.log_level
    }

    pub fn set_log_level(&mut self, log_level: i32) {
        self.log_level = log_level;
    }

    pub fn is_logging(&self, log_level: i32) -> bool {
        self.log_level >= log_level
    }

    pub fn force_log(&mut self, game_id: i64) {
        self.force_log.insert(game_id);
    }

    pub fn log(&self, log_level: i32, game_id: i64, log_string: &str) {
        if (self.is_logging(log_level) || self.force_log.contains(&game_id)) && !log_string.is_empty() {
            self.log_internal(Some(game_id), None, None, log_string);
        }
    }

    pub fn log_with_out_game_id(&self, log_level: i32, log_string: &str) {
        if self.is_logging(log_level) && !log_string.is_empty() {
            self.log_internal(Some(-1), None, None, log_string);
        }
    }

    pub fn log_replay(&self, log_level: i32, replay_name: &str, flag: Option<&str>, log_string: &str) {
        if self.is_logging(log_level) && !log_string.is_empty() {
            self.log_internal(None, Some(replay_name), flag, log_string);
        }
    }

    fn log_internal(&self, game_id: Option<i64>, replay_name: Option<&str>, command_flag: Option<&str>, log_string: &str) {
        // Phase ZU: wire actual file I/O
        let _ = (game_id, replay_name, command_flag, log_string);
    }

    pub fn close_resources(&mut self, id: i64) {
        self.log_files.remove(&id);
    }
}

impl Default for DebugLog {
    fn default() -> Self {
        Self::new(PathBuf::from("ffb.log"), PathBuf::from("logs"), IServerLogLevel::WARN, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_logging_respects_level() {
        let log = DebugLog::default();
        assert!(log.is_logging(IServerLogLevel::ERROR));
        assert!(log.is_logging(IServerLogLevel::WARN));
        assert!(!log.is_logging(IServerLogLevel::INFO));
    }

    #[test]
    fn test_command_flag_constants() {
        assert_eq!(DebugLog::COMMAND_CLIENT_HOME, " H->");
        assert_eq!(DebugLog::COMMAND_SERVER_HOME, " ->H");
    }
}
