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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_message() {
        let e = FantasyFootballException::new("test error".to_string());
        assert_eq!(e.get_message(), "test error");
    }

    #[test]
    fn display_contains_message() {
        let e = FantasyFootballException::new("boom".to_string());
        assert!(e.to_string().contains("boom"));
    }
}
