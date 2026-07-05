/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandStatus`.
/// Reports server connection status (error or success) to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandStatus {
    /// Java: `fServerStatus` — status code; stored as name string (ServerStatus not yet exported).
    pub server_status: String,
    /// Java: `fMessage` — human-readable status message.
    pub message: String,
}

impl ServerCommandStatus {
    pub fn new(server_status: impl Into<String>, message: impl Into<String>) -> Self {
        Self { server_status: server_status.into(), message: message.into() }
    }
    pub fn get_server_status(&self) -> &str { &self.server_status }
    pub fn get_message(&self) -> &str { &self.message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandStatus::new("OK", "Connected");
        assert_eq!(cmd.get_server_status(), "OK");
        assert_eq!(cmd.get_message(), "Connected");
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandStatus::default();
        assert!(cmd.server_status.is_empty());
    }
}
