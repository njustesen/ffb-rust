use std::sync::Mutex;

/// Append-only log of replayable server commands for a game — 1:1 translation of Java GameLog.
pub struct GameLog {
    server_commands: Mutex<Vec<String>>,
    last_committed_command_nr: Mutex<i32>,
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            server_commands: Mutex::new(Vec::new()),
            last_committed_command_nr: Mutex::new(0),
        }
    }

    pub fn add(&self, server_command: String) {
        let mut cmds = self.server_commands.lock().unwrap();
        cmds.push(server_command);
    }

    pub fn get_server_commands(&self) -> Vec<String> {
        self.server_commands.lock().unwrap().clone()
    }

    pub fn get_uncommitted_server_commands(&self) -> Vec<String> {
        // Phase ZU: filter by command number
        todo!("Phase ZU: filter by command_nr > last_committed_command_nr")
    }

    pub fn find_max_command_nr(&self) -> i32 {
        // Phase ZU: iterate commands by command_nr field
        todo!("Phase ZU: scan command_nr fields")
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

    #[test]
    fn test_add_and_size() {
        let log = GameLog::new();
        assert_eq!(log.size(), 0);
        log.add("cmd1".to_string());
        log.add("cmd2".to_string());
        assert_eq!(log.size(), 2);
    }

    #[test]
    fn test_clear() {
        let log = GameLog::new();
        log.add("cmd".to_string());
        log.clear();
        assert_eq!(log.size(), 0);
    }
}
