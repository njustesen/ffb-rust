use std::collections::HashMap;
use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandVersion`.
/// Sends server/client version information and capabilities to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandVersion {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
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
            command_nr: 0,
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

    /// Java: `ServerCommandVersion.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("serverVersion".to_string(), serde_json::json!(self.server_version));
        map.insert("clientVersion".to_string(), serde_json::json!(self.client_version));
        let names: Vec<&String> = self.client_properties.keys().collect();
        let values: Vec<&String> = names.iter().map(|n| &self.client_properties[*n]).collect();
        map.insert("clientPropertyNames".to_string(), serde_json::json!(names));
        map.insert("clientPropertyValues".to_string(), serde_json::json!(values));
        map.insert("testing".to_string(), serde_json::json!(self.is_test_server));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandVersion.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let server_version = json.get("serverVersion").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let client_version = json.get("clientVersion").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let names: Vec<String> = json
            .get("clientPropertyNames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let values: Vec<String> = json
            .get("clientPropertyValues")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let mut client_properties = HashMap::new();
        for (name, value) in names.into_iter().zip(values.into_iter()) {
            client_properties.insert(name, value);
        }
        let is_test_server = json.get("testing").and_then(|v| v.as_bool()).unwrap_or(false);
        Self {
            command_nr: base.command_nr,
            server_version,
            client_version,
            client_properties,
            is_test_server,
        }
    }
}

impl NetCommand for ServerCommandVersion {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerVersion
    }

    fn get_context(&self) -> FactoryContext {
        FactoryContext::APPLICATION
    }
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

    #[test]
    fn get_id_is_server_version() {
        assert_eq!(ServerCommandVersion::default().get_id(), NetCommandId::ServerVersion);
    }

    #[test]
    fn get_context_is_application() {
        assert_eq!(ServerCommandVersion::default().get_context(), FactoryContext::APPLICATION);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_versions() {
        let cmd = ServerCommandVersion::new("1.2.3", "1.0.0", HashMap::new(), true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverVersion");
        assert_eq!(json["serverVersion"], "1.2.3");
        assert_eq!(json["clientVersion"], "1.0.0");
        assert_eq!(json["testing"], true);
    }

    #[test]
    fn round_trip_with_properties() {
        let mut props = HashMap::new();
        props.insert("os".to_string(), "windows".to_string());
        let mut cmd = ServerCommandVersion::new("1.2.3", "1.0.0", props, true);
        cmd.command_nr = 7;
        let json = cmd.to_json_value();
        let restored = ServerCommandVersion::from_json(&json);
        assert_eq!(restored.command_nr, 7);
        assert_eq!(restored.server_version, "1.2.3");
        assert_eq!(restored.client_version, "1.0.0");
        assert!(restored.is_test_server);
        assert_eq!(restored.get_client_properties().get("os"), Some(&"windows".to_string()));
    }

    #[test]
    fn round_trip_with_default() {
        let cmd = ServerCommandVersion::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandVersion::from_json(&json);
        assert!(restored.server_version.is_empty());
        assert!(restored.client_properties.is_empty());
        assert!(!restored.is_test_server);
    }
}
