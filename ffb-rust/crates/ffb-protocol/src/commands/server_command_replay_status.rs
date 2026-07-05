/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandReplayStatus`.
/// Communicates replay playback state to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandReplayStatus {
    /// Java: `commandNr` — current replay command index.
    pub command_nr: i32,
    /// Java: `speed` — playback speed multiplier.
    pub speed: i32,
    /// Java: `running` — whether replay is actively playing.
    pub running: bool,
    /// Java: `forward` — playback direction (true = forward).
    pub forward: bool,
    /// Java: `skip` — whether skipping to a position.
    pub skip: bool,
}

impl ServerCommandReplayStatus {
    pub fn new(command_nr: i32, speed: i32, running: bool, forward: bool, skip: bool) -> Self {
        Self { command_nr, speed, running, forward, skip }
    }
    pub fn get_command_nr(&self) -> i32 { self.command_nr }
    pub fn get_speed(&self) -> i32 { self.speed }
    pub fn is_running(&self) -> bool { self.running }
    pub fn is_forward(&self) -> bool { self.forward }
    pub fn is_skip(&self) -> bool { self.skip }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fields_stored() {
        let cmd = ServerCommandReplayStatus::new(42, 2, true, true, false);
        assert_eq!(cmd.get_command_nr(), 42);
        assert_eq!(cmd.get_speed(), 2);
        assert!(cmd.is_running());
        assert!(cmd.is_forward());
        assert!(!cmd.is_skip());
    }

    #[test]
    fn default_is_stopped() {
        let cmd = ServerCommandReplayStatus::default();
        assert!(!cmd.running);
        assert_eq!(cmd.command_nr, 0);
    }
}
