use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.ChatCommand.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatCommand {
    pub command: String,
    pub coach: String,
}

impl ChatCommand {
    pub fn new(command: String, coach: String) -> Self { Self { command, coach } }
    pub fn get_command(&self) -> &str { &self.command }
    pub fn get_coach(&self) -> &str { &self.coach }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(ChatCommand::default().command.is_empty());
    }

    #[test]
    fn get_command_returns_command() {
        let c = ChatCommand::new("/help".to_string(), "Bob".to_string());
        assert_eq!(c.get_command(), "/help");
    }
}
