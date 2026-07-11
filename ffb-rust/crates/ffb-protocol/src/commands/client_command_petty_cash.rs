use ffb_model::enums::NetCommandId;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of ClientCommandPettyCash (Java field: fPettyCash).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPettyCash {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    pub petty_cash: i32,
}

impl ClientCommandPettyCash {
    pub fn new(petty_cash: i32) -> Self {
        Self { entropy: None, petty_cash }
    }

    pub fn get_petty_cash(&self) -> i32 {
        self.petty_cash
    }

    /// Java: `ClientCommandPettyCash.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("pettyCash".to_string(), serde_json::json!(self.petty_cash));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandPettyCash.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            petty_cash: json.get("pettyCash").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
        }
    }
}

impl NetCommand for ClientCommandPettyCash {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientPettyCash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value() {
        let cmd = ClientCommandPettyCash::new(50_000);
        assert_eq!(cmd.get_petty_cash(), 50_000);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandPettyCash::default();
        assert_eq!(cmd.get_petty_cash(), 0);
    }

    #[test]
    fn negative_value_stored() {
        let cmd = ClientCommandPettyCash::new(-1000);
        assert_eq!(cmd.get_petty_cash(), -1000);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPettyCash::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPettyCash::default().clone();
    }

    #[test]
    fn get_id_is_client_petty_cash() {
        assert_eq!(ClientCommandPettyCash::default().get_id(), NetCommandId::ClientPettyCash);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_petty_cash() {
        let cmd = ClientCommandPettyCash::new(1234);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientPettyCash");
        assert_eq!(json["pettyCash"], 1234);
    }

    #[test]
    fn round_trip_with_data() {
        let mut cmd = ClientCommandPettyCash::new(-500);
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandPettyCash::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.get_petty_cash(), -500);
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandPettyCash::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandPettyCash::from_json(&json);
        assert_eq!(restored.petty_cash, 0);
        assert!(restored.entropy.is_none());
    }
}
