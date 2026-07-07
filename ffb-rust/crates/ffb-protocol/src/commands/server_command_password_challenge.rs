/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandPasswordChallenge`.
/// Sends a password challenge string to the client for authentication.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandPasswordChallenge {
    /// Java: `fChallenge` — the challenge string.
    pub challenge: String,
}

impl ServerCommandPasswordChallenge {
    pub fn new(challenge: impl Into<String>) -> Self {
        Self { challenge: challenge.into() }
    }
    pub fn get_challenge(&self) -> &str { &self.challenge }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_stored() {
        let cmd = ServerCommandPasswordChallenge::new("abc123");
        assert_eq!(cmd.get_challenge(), "abc123");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandPasswordChallenge::default();
        assert!(cmd.challenge.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandPasswordChallenge::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandPasswordChallenge::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandPasswordChallenge::default());
        assert!(s.contains("ServerCommandPasswordChallenge"));
    }
}
