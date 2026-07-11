/// 1:1 translation of com.fumbbl.ffb.server.net.ReceivedCommand.
/// Wraps a net command with the session it arrived on.
pub struct ReceivedCommand {
    /// Serialized command id string (from NetCommandId).
    pub command_id: String,
    /// Whether the command is an internal server command.
    pub internal: bool,
    /// Whether the command is a client command.
    pub client: bool,
    /// Opaque session identifier.
    pub session_id: u64,
}

impl ReceivedCommand {
    pub fn new(command_id: String, internal: bool, client: bool, session_id: u64) -> Self {
        Self { command_id, internal, client, session_id }
    }

    pub fn get_id(&self) -> &str {
        &self.command_id
    }

    pub fn is_internal_command(&self) -> bool {
        self.internal
    }

    pub fn is_client_command(&self) -> bool {
        self.client
    }

    pub fn get_session(&self) -> u64 {
        self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = ReceivedCommand::new("testCmd".to_string(), false, true, 1);
    }

    #[test]
    fn get_id() {
        let rc = ReceivedCommand::new("myCmd".to_string(), false, true, 42);
        assert_eq!(rc.get_id(), "myCmd");
    }

    #[test]
    fn is_internal_command() {
        let rc = ReceivedCommand::new("x".to_string(), true, false, 1);
        assert!(rc.is_internal_command());
    }

    #[test]
    fn is_client_command() {
        let rc = ReceivedCommand::new("x".to_string(), false, true, 1);
        assert!(rc.is_client_command());
    }

    #[test]
    fn get_session() {
        let rc = ReceivedCommand::new("x".to_string(), false, false, 99);
        assert_eq!(rc.get_session(), 99);
    }
}
