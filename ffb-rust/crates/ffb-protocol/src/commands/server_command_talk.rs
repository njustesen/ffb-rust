/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTalk`.
/// Delivers chat messages from coaches to all clients.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandTalk {
    /// Java: `fCoach` — sending coach name.
    pub coach: String,
    /// Java: `fTalks` — list of chat message strings.
    pub talks: Vec<String>,
    /// Java: `mode` — chat mode (REGULAR, WHISPER, etc.); stored as name string.
    pub mode: String,
}

impl ServerCommandTalk {
    pub fn new(coach: impl Into<String>, talks: Vec<String>, mode: impl Into<String>) -> Self {
        Self { coach: coach.into(), talks, mode: mode.into() }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_talks(&self) -> &[String] { &self.talks }
    pub fn get_mode(&self) -> &str { &self.mode }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTalk::new("Alice", vec!["hi".into()], "REGULAR");
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_talks(), &["hi"]);
        assert_eq!(cmd.get_mode(), "REGULAR");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandTalk::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.talks.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandTalk::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandTalk::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandTalk::default());
        assert!(s.contains("ServerCommandTalk"));
    }
}
