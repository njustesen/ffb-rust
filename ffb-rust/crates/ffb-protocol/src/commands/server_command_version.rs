use std::collections::HashMap;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandVersion`.
/// Sends server/client version information and capabilities to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandVersion {
    /// Java: `fServerVersion` — server software version string.
    pub server_version: String,
    /// Java: `fClientVersion` — minimum required client version.
    pub client_version: String,
    /// Java: `fClientProperties` — key-value capability map.
    pub client_properties: HashMap<String, String>,
    /// Java: `isTestServer` — true if running in test mode.
    pub is_test_server: bool,
}

impl ServerCommandVersion {
    pub fn new(
        server_version: impl Into<String>,
        client_version: impl Into<String>,
        client_properties: HashMap<String, String>,
        is_test_server: bool,
    ) -> Self {
        Self {
            server_version: server_version.into(),
            client_version: client_version.into(),
            client_properties,
            is_test_server,
        }
    }
    pub fn get_server_version(&self) -> &str { &self.server_version }
    pub fn get_client_version(&self) -> &str { &self.client_version }
    pub fn get_client_properties(&self) -> &HashMap<String, String> { &self.client_properties }
    pub fn is_test_server(&self) -> bool { self.is_test_server }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandVersion::new("1.2.3", "1.0.0", HashMap::new(), false);
        assert_eq!(cmd.get_server_version(), "1.2.3");
        assert_eq!(cmd.get_client_version(), "1.0.0");
        assert!(!cmd.is_test_server());
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandVersion::default();
        assert!(cmd.server_version.is_empty());
        assert!(cmd.client_properties.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandVersion::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandVersion::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandVersion::default());
        assert!(s.contains("ServerCommandVersion"));
    }
}
