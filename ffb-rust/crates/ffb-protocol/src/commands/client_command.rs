use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommand`.
/// Base class for all client commands — carries optional entropy byte for anti-replay.
#[derive(Debug, Clone, Default)]
pub struct ClientCommand {
    /// Java: `fEntropy` — optional entropy byte for anti-replay protection.
    pub entropy: Option<u8>,
}

impl ClientCommand {
    pub fn new() -> Self { Self::default() }
    pub fn get_entropy(&self) -> Option<u8> { self.entropy }
    pub fn has_entropy(&self) -> bool { self.entropy.is_some() }
    pub fn set_entropy(&mut self, entropy: u8) { self.entropy = Some(entropy); }

    /// Java: `ClientCommand.toJsonValue()` — the base-class fields every
    /// subclass's `toJsonValue()` starts from (`super.toJsonValue()`).
    pub fn base_json_fields(&self, id: NetCommandId) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("netCommandId".to_string(), serde_json::json!(id.name()));
        if let Some(entropy) = self.entropy {
            map.insert("entropy".to_string(), serde_json::json!(entropy));
        }
        map
    }

    /// Java: `ClientCommand.initFrom(source, jsonValue)` — reads only the
    /// base-class fields; the `netCommandId` key itself is consumed by the
    /// factory dispatch, not re-parsed here (matches `UtilNetCommand
    /// .validateCommandId`'s role of checking, not extracting, the id).
    pub fn base_from_json(json: &serde_json::Value) -> Self {
        Self {
            entropy: json.get("entropy").and_then(|v| v.as_u64()).map(|v| v as u8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_no_entropy() {
        assert!(ClientCommand::new().entropy.is_none());
    }
    #[test]
    fn entropy_stored() {
        let cmd = ClientCommand { entropy: Some(42) };
        assert_eq!(cmd.get_entropy(), Some(42));
    }

    #[test]
    fn max_entropy_stored() {
        let cmd = ClientCommand { entropy: Some(255) };
        assert_eq!(cmd.get_entropy(), Some(255));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommand::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommand::default().clone();
    }

    #[test]
    fn has_entropy_reflects_state() {
        assert!(!ClientCommand::new().has_entropy());
        let mut cmd = ClientCommand::new();
        cmd.set_entropy(9);
        assert!(cmd.has_entropy());
    }

    #[test]
    fn base_json_fields_includes_net_command_id() {
        let cmd = ClientCommand::new();
        let fields = cmd.base_json_fields(NetCommandId::ClientJoin);
        assert_eq!(fields["netCommandId"], "clientJoin");
        assert!(!fields.contains_key("entropy"));
    }

    #[test]
    fn base_json_fields_includes_entropy_when_present() {
        let mut cmd = ClientCommand::new();
        cmd.set_entropy(7);
        let fields = cmd.base_json_fields(NetCommandId::ClientJoin);
        assert_eq!(fields["entropy"], 7);
    }

    #[test]
    fn base_from_json_round_trip() {
        let mut cmd = ClientCommand::new();
        cmd.set_entropy(3);
        let json = serde_json::Value::Object(cmd.base_json_fields(NetCommandId::ClientJoin));
        let restored = ClientCommand::base_from_json(&json);
        assert_eq!(restored.entropy, Some(3));
    }
}
