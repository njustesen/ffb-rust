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
