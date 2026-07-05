/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandReplayControl`.
/// Tells the client which coach is controlling replay playback.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandReplayControl {
    /// Java: `coach` — coach name who controls the replay.
    pub coach: String,
}

impl ServerCommandReplayControl {
    pub fn new(coach: impl Into<String>) -> Self { Self { coach: coach.into() } }
    pub fn get_coach(&self) -> &str { &self.coach }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coach_stored() {
        let cmd = ServerCommandReplayControl::new("Alice");
        assert_eq!(cmd.get_coach(), "Alice");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandReplayControl::default();
        assert!(cmd.coach.is_empty());
    }
}
