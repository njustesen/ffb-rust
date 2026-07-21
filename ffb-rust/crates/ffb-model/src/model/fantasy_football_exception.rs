use std::fmt;

/// 1:1 translation of com.fumbbl.ffb.FantasyFootballException.
#[derive(Debug, Clone)]
pub struct FantasyFootballException {
    pub message: String,
}

impl FantasyFootballException {
    pub fn new(message: String) -> Self { Self { message } }
    pub fn get_message(&self) -> &str { &self.message }
}

impl fmt::Display for FantasyFootballException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FantasyFootballException: {}", self.message)
    }
}

impl std::error::Error for FantasyFootballException {}
