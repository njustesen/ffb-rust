/// A replay session with server commands to be replayed — 1:1 translation of Java ServerReplay.
pub struct ServerReplay {
    server_commands: Vec<String>,
    game_id: i64,
    from_command_nr: i32,
    to_command_nr: i32,
    complete: bool,
}

impl ServerReplay {
    pub fn new(game_id: i64, to_command_nr: i32) -> Self {
        Self {
            server_commands: Vec::new(),
            game_id,
            from_command_nr: 0,
            to_command_nr,
            complete: false,
        }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_from_command_nr(&self) -> i32 {
        self.from_command_nr
    }

    pub fn set_from_command_nr(&mut self, nr: i32) {
        self.from_command_nr = nr;
    }

    pub fn get_to_command_nr(&self) -> i32 {
        self.to_command_nr
    }

    pub fn get_server_commands(&self) -> &[String] {
        &self.server_commands
    }

    pub fn size(&self) -> usize {
        if self.to_command_nr == 0 {
            self.server_commands.len()
        } else {
            (self.to_command_nr - 1).max(0) as usize
        }
    }

    pub fn set_complete(&mut self, complete: bool) {
        self.complete = complete;
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn add_command(&mut self, cmd: String) {
        self.server_commands.push(cmd);
    }
}

impl Default for ServerReplay {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_replay_not_complete() {
        let replay = ServerReplay::new(42, 100);
        assert!(!replay.is_complete());
        assert_eq!(replay.get_game_id(), 42);
        assert_eq!(replay.get_to_command_nr(), 100);
    }

    #[test]
    fn test_size_with_to_command_nr() {
        let replay = ServerReplay::new(1, 5);
        assert_eq!(replay.size(), 4);
    }
}
