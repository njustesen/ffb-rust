/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandReplayStatus`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandReplayStatus {
    /// Java: `commandNr`
    pub command_nr: i32,
    /// Java: `speed`
    pub speed: i32,
    /// Java: `running`
    pub running: bool,
    /// Java: `forward`
    pub forward: bool,
    /// Java: `skip`
    pub skip: bool,
}

impl ClientCommandReplayStatus {
    pub fn new() -> Self { Self::default() }

    pub fn with_params(command_nr: i32, speed: i32, running: bool, forward: bool, skip: bool) -> Self {
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
    fn fields_stored() {
        let cmd = ClientCommandReplayStatus::with_params(55, 2, true, true, false);
        assert_eq!(cmd.get_command_nr(), 55);
        assert_eq!(cmd.get_speed(), 2);
        assert!(cmd.is_running());
        assert!(cmd.is_forward());
        assert!(!cmd.is_skip());
    }

    #[test]
    fn default_is_zeroed() {
        let cmd = ClientCommandReplayStatus::new();
        assert_eq!(cmd.command_nr, 0);
        assert!(!cmd.running);
        assert!(!cmd.skip);
    }

    #[test]
    fn skip_can_be_set() {
        let cmd = ClientCommandReplayStatus::with_params(0, 1, false, false, true);
        assert!(cmd.is_skip());
        assert!(!cmd.is_running());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandReplayStatus::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandReplayStatus::default().clone();
    }
}
