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
