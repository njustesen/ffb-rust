use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPuntToCrowd`.
/// Sent when a player punts the ball to the crowd.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPuntToCrowd {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `puntToCrowd`
    pub punt_to_crowd: bool,
}

impl ClientCommandPuntToCrowd {
    pub fn new(punt_to_crowd: bool) -> Self { Self { entropy: None, punt_to_crowd } }
    pub fn is_punt_to_crowd(&self) -> bool { self.punt_to_crowd }

    /// Java: `ClientCommandPuntToCrowd.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("puntToCrowd".to_string(), serde_json::json!(self.punt_to_crowd));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPuntToCrowd.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            punt_to_crowd: json.get("puntToCrowd").and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }
}

impl NetCommand for ClientCommandPuntToCrowd {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPuntToCrowd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn true_stored() {
        let cmd = ClientCommandPuntToCrowd::new(true);
        assert!(cmd.is_punt_to_crowd());
    }
    #[test]
    fn default_false() {
        let cmd = ClientCommandPuntToCrowd::default();
        assert!(!cmd.punt_to_crowd);
    }

    #[test]
    fn false_stored() {
        let cmd = ClientCommandPuntToCrowd::new(false);
        assert!(!cmd.is_punt_to_crowd());
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPuntToCrowd::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPuntToCrowd::default().clone();
    }

    #[test]
    fn get_id_is_client_punt_to_crowd() {
        assert_eq!(ClientCommandPuntToCrowd::default().get_id(), NetCommandId::ClientPuntToCrowd);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_punt_to_crowd() {
        let cmd = ClientCommandPuntToCrowd::new(true);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPuntToCrowd");
        assert_eq!(json["puntToCrowd"], true);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPuntToCrowd::new(true);
        cmd.entropy = Some(6);
        let json = cmd.to_json_value();
        let restored = ClientCommandPuntToCrowd::from_json(&json);
        assert_eq!(restored.entropy, Some(6));
        assert!(restored.is_punt_to_crowd());
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPuntToCrowd::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPuntToCrowd::from_json(&json);
        assert!(!restored.is_punt_to_crowd());
        assert!(restored.entropy.is_none());
    }
}
