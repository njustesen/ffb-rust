use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.PasswordChallenge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PasswordChallenge {
    pub challenge: String,
}

impl PasswordChallenge {
    pub fn new(challenge: String) -> Self { Self { challenge } }
    pub fn get_challenge(&self) -> &str { &self.challenge }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(PasswordChallenge::default().challenge.is_empty());
    }

    #[test]
    fn new_sets_challenge() {
        let pc = PasswordChallenge::new("abc123".to_string());
        assert_eq!(pc.get_challenge(), "abc123");
    }
}
