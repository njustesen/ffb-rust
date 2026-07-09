use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.NetCommandLog`.
/// Accumulates `NetCommand` values for logging / replay purposes.
#[derive(Debug, Default)]
pub struct NetCommandLog {
    commands: Vec<NetCommand>,
}

impl NetCommandLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a command to the log.
    pub fn add(&mut self, command: NetCommand) {
        self.commands.push(command);
    }

    /// Return a slice of all logged commands.
    pub fn commands(&self) -> &[NetCommand] {
        &self.commands
    }

    /// Number of commands in the log.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_is_empty() {
        let log = NetCommandLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn add_increments_len() {
        let mut log = NetCommandLog::new();
        log.add(NetCommand::Unknown);
        log.add(NetCommand::Unknown);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn commands_returns_slice() {
        let mut log = NetCommandLog::new();
        log.add(NetCommand::Unknown);
        assert_eq!(log.commands().len(), 1);
    }
}
