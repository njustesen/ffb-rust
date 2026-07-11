use crate::game_log::GameLog;

/// A replay session with server commands to be replayed — 1:1 translation of Java ServerReplay.
///
/// Java's constructor takes a `GameState` (from which it reads `getId()` and
/// `getGameLog().getServerCommands()`) and a Jetty `Session`. This crate's `GameLog` lives
/// here in `ffb-engine` (unlike `GameState`, which is a `ffb-server`-only wrapper), so the
/// `GameLog` reference is taken directly rather than threading a whole `GameState` through;
/// `session` is a plain `u64` session id standing in for the Jetty `Session` object (same
/// substitution `ReplaySessionManager`/`SessionManager` already make elsewhere).
///
/// Java's `orderCommands()` renumbers every command stored in the array in place
/// (`fServerCommands[i].setCommandNr(i + 1)`), then later filters by that renumbered
/// `commandNr`. `AnyServerCommand` (the type `GameLog` stores) isn't `Clone` and is only
/// reachable through `GameLog`'s `Mutex` guard, so this crate can't hold onto or mutate the
/// original command objects; instead each command is serialized to its JSON wire form once
/// (via `AnyServerCommand::to_json_value`), with `commandNr` overwritten to its renumbered
/// value in that JSON copy. Because renumbering is always `index + 1`, the renumbered
/// command_nr for each stored string is simply its position — no separate command_nr needs
/// to be tracked alongside the string.
pub struct ServerReplay {
    server_commands: Vec<String>,
    game_id: i64,
    from_command_nr: i32,
    to_command_nr: i32,
    session: u64,
    complete: bool,
}

impl ServerReplay {
    /// Java: `ServerReplay(GameState gameState, int toCommandNr, Session session)`.
    pub fn new(game_id: i64, to_command_nr: i32, session: u64, game_log: &GameLog) -> Self {
        Self {
            server_commands: order_commands(game_log),
            game_id,
            from_command_nr: 0,
            to_command_nr,
            session,
            complete: false,
        }
    }

    /// Not a Java method — constructs a `ServerReplay` with no backing `GameLog`, for tests
    /// that only need the from/to/size bookkeeping and not real replayed commands.
    #[cfg(test)]
    pub fn empty(game_id: i64, to_command_nr: i32, session: u64) -> Self {
        Self {
            server_commands: Vec::new(),
            game_id,
            from_command_nr: 0,
            to_command_nr,
            session,
            complete: false,
        }
    }

    pub fn get_game_id(&self) -> i64 {
        self.game_id
    }

    pub fn get_session(&self) -> u64 {
        self.session
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

    #[cfg(test)]
    pub fn add_command(&mut self, cmd: String) {
        self.server_commands.push(cmd);
    }

    /// Java: `findRelevantCommandsInLog()`. Each stored command's renumbered `commandNr`
    /// equals its 1-based position (see the struct doc comment), so filtering operates on
    /// the index directly rather than re-parsing each JSON string.
    pub fn find_relevant_commands_in_log(&self) -> Vec<&str> {
        self.server_commands
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let command_nr = (*i + 1) as i32;
                command_nr >= self.from_command_nr
                    && (self.to_command_nr == 0 || command_nr < self.to_command_nr)
            })
            .map(|(_, s)| s.as_str())
            .collect()
    }
}

/// Java: `ServerReplay.orderCommands()`.
fn order_commands(game_log: &GameLog) -> Vec<String> {
    let commands = game_log.get_server_commands();
    commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let mut json = cmd.to_json_value();
            if let Some(obj) = json.as_object_mut() {
                obj.insert("commandNr".to_string(), serde_json::json!((i + 1) as i32));
            }
            json.to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_protocol::commands::any_server_command::AnyServerCommand;
    use ffb_protocol::commands::server_command_pong::ServerCommandPong;

    fn pong_cmd(command_nr: i32) -> AnyServerCommand {
        let mut cmd = ServerCommandPong::default();
        cmd.command_nr = command_nr;
        AnyServerCommand::ServerPong(cmd)
    }

    #[test]
    fn test_new_replay_not_complete() {
        let replay = ServerReplay::empty(42, 100, 1);
        assert!(!replay.is_complete());
        assert_eq!(replay.get_game_id(), 42);
        assert_eq!(replay.get_to_command_nr(), 100);
        assert_eq!(replay.get_session(), 1);
    }

    #[test]
    fn test_size_with_to_command_nr() {
        let replay = ServerReplay::empty(1, 5, 1);
        assert_eq!(replay.size(), 4);
    }

    #[test]
    fn new_from_game_log_orders_and_renumbers_commands() {
        let log = GameLog::new();
        log.add(pong_cmd(1));
        log.add(pong_cmd(2));
        log.add(pong_cmd(3));
        let replay = ServerReplay::new(7, 0, 1, &log);
        assert_eq!(replay.get_server_commands().len(), 3);
        // Each stored command is renumbered to its 1-based position, regardless of the
        // command_nr it carried in the log.
        let first: serde_json::Value = serde_json::from_str(&replay.get_server_commands()[0]).unwrap();
        assert_eq!(first["commandNr"], 1);
        let third: serde_json::Value = serde_json::from_str(&replay.get_server_commands()[2]).unwrap();
        assert_eq!(third["commandNr"], 3);
    }

    #[test]
    fn find_relevant_commands_in_log_filters_by_from_and_to() {
        let log = GameLog::new();
        for i in 1..=5 {
            log.add(pong_cmd(i));
        }
        let mut replay = ServerReplay::new(1, 4, 1, &log);
        replay.set_from_command_nr(2);
        let relevant = replay.find_relevant_commands_in_log();
        // commandNr >= 2 && commandNr < 4 -> positions 2 and 3.
        assert_eq!(relevant.len(), 2);
        let first: serde_json::Value = serde_json::from_str(relevant[0]).unwrap();
        assert_eq!(first["commandNr"], 2);
        let second: serde_json::Value = serde_json::from_str(relevant[1]).unwrap();
        assert_eq!(second["commandNr"], 3);
    }

    #[test]
    fn find_relevant_commands_in_log_with_zero_to_command_nr_takes_all_from_start() {
        let log = GameLog::new();
        for i in 1..=3 {
            log.add(pong_cmd(i));
        }
        let replay = ServerReplay::new(1, 0, 1, &log);
        assert_eq!(replay.find_relevant_commands_in_log().len(), 3);
    }

    #[test]
    fn add_command_appends_to_server_commands() {
        let mut replay = ServerReplay::empty(1, 0, 1);
        replay.add_command("{\"commandNr\":1}".to_string());
        assert_eq!(replay.get_server_commands().len(), 1);
    }
}
