use std::sync::Mutex;

use ffb_protocol::commands::any_server_command::AnyServerCommand;

/// Append-only log of replayable server commands for a game — 1:1 translation of Java GameLog.
pub struct GameLog {
    server_commands: Mutex<Vec<AnyServerCommand>>,
    last_committed_command_nr: Mutex<i32>,
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            server_commands: Mutex::new(Vec::new()),
            last_committed_command_nr: Mutex::new(0),
        }
    }

    /// Java: `add(ServerCommand pServerCommand)` — only replayable commands are kept.
    pub fn add(&self, server_command: AnyServerCommand) {
        if server_command.is_replayable() {
            let mut cmds = self.server_commands.lock().unwrap();
            cmds.push(server_command);
        }
    }

    /// Java: `getServerCommands()` — returns all stored commands.
    ///
    /// `AnyServerCommand` is not `Clone` (see its own doc comment), so a Java-style
    /// `ServerCommand[]` snapshot copy isn't available; instead this returns the
    /// locked `MutexGuard`, which derefs to `&[AnyServerCommand]` and gives callers
    /// direct read access to the same underlying storage Java would have copied from.
    pub fn get_server_commands(&self) -> std::sync::MutexGuard<'_, Vec<AnyServerCommand>> {
        self.server_commands.lock().unwrap()
    }

    /// Java: `getUncommitedServerCommands()`.
    ///
    /// Returns the `command_nr` of every stored command with
    /// `command_nr > getLastCommitedCommandNr()`, matching the Java filter. Returns
    /// command numbers rather than command references because `AnyServerCommand` is
    /// not `Clone` and nothing downstream currently needs the full command payloads —
    /// callers needing those can use `get_server_commands()` and re-filter directly.
    pub fn get_uncommitted_server_commands(&self) -> Vec<i32> {
        let last_committed = self.get_last_committed_command_nr();
        let cmds = self.server_commands.lock().unwrap();
        cmds.iter()
            .map(|c| c.get_command_nr())
            .filter(|nr| *nr > last_committed)
            .collect()
    }

    /// Java: `findMaxCommandNr()`.
    pub fn find_max_command_nr(&self) -> i32 {
        let cmds = self.server_commands.lock().unwrap();
        let mut max_command_nr = 0;
        for server_command in cmds.iter() {
            if server_command.get_command_nr() > max_command_nr {
                max_command_nr = server_command.get_command_nr();
            }
        }
        max_command_nr
    }

    pub fn set_last_committed_command_nr(&self, nr: i32) {
        *self.last_committed_command_nr.lock().unwrap() = nr;
    }

    pub fn get_last_committed_command_nr(&self) -> i32 {
        *self.last_committed_command_nr.lock().unwrap()
    }

    pub fn clear(&self) {
        self.server_commands.lock().unwrap().clear();
    }

    pub fn size(&self) -> usize {
        self.server_commands.lock().unwrap().len()
    }
}

impl Default for GameLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::commands::server_command_game_time::ServerCommandGameTime;
    use ffb_protocol::commands::server_command_pong::ServerCommandPong;

    fn game_time_cmd(command_nr: i32) -> AnyServerCommand {
        // ServerCommandGameTime::is_replayable() returns false, so it's a useful
        // stand-in for verifying the non-replayable guard in `add`.
        let mut cmd = ServerCommandGameTime::new(1, 2);
        cmd.command_nr = command_nr;
        AnyServerCommand::ServerGameTime(cmd)
    }

    fn pong_cmd(command_nr: i32) -> AnyServerCommand {
        // ServerCommandPong has no isReplayable() override, so it's replayable
        // (inherits the ServerCommand base default of true).
        let mut cmd = ServerCommandPong::default();
        cmd.command_nr = command_nr;
        AnyServerCommand::ServerPong(cmd)
    }

    #[test]
    fn test_add_and_size() {
        let log = GameLog::new();
        assert_eq!(log.size(), 0);
        log.add(pong_cmd(1));
        log.add(pong_cmd(2));
        assert_eq!(log.size(), 2);
    }

    #[test]
    fn add_skips_non_replayable_commands() {
        let log = GameLog::new();
        log.add(game_time_cmd(1));
        assert_eq!(log.size(), 0);
    }

    #[test]
    fn add_keeps_replayable_commands() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        assert_eq!(log.size(), 1);
    }

    #[test]
    fn test_clear() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.clear();
        assert_eq!(log.size(), 0);
    }

    #[test]
    fn get_uncommitted_server_commands_filters_by_last_committed() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.add(pong_cmd(2));
        log.add(pong_cmd(3));
        log.set_last_committed_command_nr(1);
        assert_eq!(log.get_uncommitted_server_commands(), vec![2, 3]);
    }

    #[test]
    fn get_uncommitted_server_commands_empty_when_all_committed() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.set_last_committed_command_nr(5);
        assert!(log.get_uncommitted_server_commands().is_empty());
    }

    #[test]
    fn find_max_command_nr_returns_zero_for_empty_log() {
        let log = GameLog::new();
        assert_eq!(log.find_max_command_nr(), 0);
    }

    #[test]
    fn find_max_command_nr_finds_the_highest_command_nr() {
        let log = GameLog::new();
        log.add(pong_cmd(3));
        log.add(pong_cmd(7));
        log.add(pong_cmd(2));
        assert_eq!(log.find_max_command_nr(), 7);
    }

    #[test]
    fn set_and_get_last_committed_command_nr() {
        let log = GameLog::new();
        assert_eq!(log.get_last_committed_command_nr(), 0);
        log.set_last_committed_command_nr(9);
        assert_eq!(log.get_last_committed_command_nr(), 9);
    }

    #[test]
    fn get_server_commands_exposes_stored_commands() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.add(pong_cmd(2));
        let cmds = log.get_server_commands();
        let nrs: Vec<i32> = cmds.iter().map(|c| c.get_command_nr()).collect();
        assert_eq!(nrs, vec![1, 2]);
    }
}
