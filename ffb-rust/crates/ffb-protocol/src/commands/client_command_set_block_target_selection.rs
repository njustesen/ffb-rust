use ffb_model::model::BlockKind;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSetBlockTargetSelection`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSetBlockTargetSelection {
    /// Java: `playerId`
    pub player_id: Option<String>,
    /// Java: `kind`
    pub kind: Option<BlockKind>,
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
}

impl ClientCommandSetBlockTargetSelection {
    pub fn new() -> Self { Self::default() }

    pub fn with_target(player_id: impl Into<String>, kind: BlockKind) -> Self {
        Self { player_id: Some(player_id.into()), kind: Some(kind), entropy: None }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_kind(&self) -> Option<BlockKind> { self.kind }

    /// Java: `ClientCommandSetBlockTargetSelection.toJsonValue()`. Java calls
    /// `kind.name()` unconditionally (would NPE on a null `kind`); here we
    /// only emit the field when `kind` is `Some` to avoid panicking.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        map.insert("playerId".to_string(), match &self.player_id {
            Some(s) => serde_json::json!(s),
            None => serde_json::Value::Null,
        });
        if let Some(kind) = self.kind {
            map.insert("blockKind".to_string(), serde_json::to_value(kind).unwrap());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSetBlockTargetSelection.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let kind = json.get("blockKind").and_then(|v| serde_json::from_value::<BlockKind>(v.clone()).ok());
        Self {
            player_id: json.get("playerId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            kind,
            entropy: base.entropy,
        }
    }
}

impl NetCommand for ClientCommandSetBlockTargetSelection {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSetBlockTargetSelection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandSetBlockTargetSelection::with_target("p1", BlockKind::BLOCK);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_kind(), Some(BlockKind::BLOCK));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSetBlockTargetSelection::new();
        assert!(cmd.player_id.is_none());
        assert!(cmd.kind.is_none());
    }

    #[test]
    fn stab_kind_stored() {
        let cmd = ClientCommandSetBlockTargetSelection::with_target("p2", BlockKind::STAB);
        assert_eq!(cmd.get_kind(), Some(BlockKind::STAB));
        assert_eq!(cmd.get_player_id(), Some("p2"));
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandSetBlockTargetSelection::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSetBlockTargetSelection::default().clone();
    }

    #[test]
    fn get_id_is_client_set_block_target_selection() {
        assert_eq!(
            ClientCommandSetBlockTargetSelection::new().get_id(),
            NetCommandId::ClientSetBlockTargetSelection
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_block_kind() {
        let cmd = ClientCommandSetBlockTargetSelection::with_target("p1", BlockKind::STAB);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSetBlockTargetSelection");
        assert_eq!(json["blockKind"], "STAB");
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn round_trip_populated() {
        let mut cmd = ClientCommandSetBlockTargetSelection::with_target("p2", BlockKind::CHAINSAW);
        cmd.entropy = Some(4);
        let json = cmd.to_json_value();
        let restored = ClientCommandSetBlockTargetSelection::from_json(&json);
        assert_eq!(restored.player_id.as_deref(), Some("p2"));
        assert_eq!(restored.kind, Some(BlockKind::CHAINSAW));
        assert_eq!(restored.entropy, Some(4));
    }

    #[test]
    fn round_trip_default() {
        let cmd = ClientCommandSetBlockTargetSelection::default();
        let json = cmd.to_json_value();
        let restored = ClientCommandSetBlockTargetSelection::from_json(&json);
        assert_eq!(restored.player_id, None);
        assert_eq!(restored.kind, None);
        assert_eq!(restored.entropy, None);
    }
}
