use ffb_model::enums::NetCommandId;
use ffb_model::model::BlockTarget;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandSynchronousMultiBlock`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandSynchronousMultiBlock {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `selectedBlockTargets`
    pub selected_block_targets: Vec<BlockTarget>,
}

impl ClientCommandSynchronousMultiBlock {
    pub fn new() -> Self { Self::default() }

    pub fn with_targets(selected_block_targets: Vec<BlockTarget>) -> Self {
        Self { entropy: None, selected_block_targets }
    }

    pub fn get_selected_block_targets(&self) -> &[BlockTarget] { &self.selected_block_targets }

    pub fn add_target(&mut self, target: BlockTarget) {
        self.selected_block_targets.push(target);
    }

    /// Java: `ClientCommandSynchronousMultiBlock.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        let targets: Vec<serde_json::Value> = self
            .selected_block_targets
            .iter()
            .map(BlockTarget::to_json_value)
            .collect();
        map.insert("selectedBlockTargets".to_string(), serde_json::Value::Array(targets));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandSynchronousMultiBlock.initFrom(source, jsonValue)`.
    /// Java caps the parsed list at 2 elements via `.limit(2)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let selected_block_targets = json
            .get("selectedBlockTargets")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().take(2).map(BlockTarget::from_json).collect())
            .unwrap_or_default();
        Self { entropy: base.entropy, selected_block_targets }
    }
}

impl NetCommand for ClientCommandSynchronousMultiBlock {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientSynchronousMultiBlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::BlockKind;

    #[test]
    fn targets_stored() {
        let t = BlockTarget::new("p1", BlockKind::BLOCK, None);
        let cmd = ClientCommandSynchronousMultiBlock::with_targets(vec![t]);
        assert_eq!(cmd.get_selected_block_targets().len(), 1);
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandSynchronousMultiBlock::new();
        assert!(cmd.selected_block_targets.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandSynchronousMultiBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandSynchronousMultiBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandSynchronousMultiBlock::default());
        assert!(s.contains("ClientCommandSynchronousMultiBlock"));
    }

    #[test]
    fn get_id_is_client_synchronous_multi_block() {
        assert_eq!(
            ClientCommandSynchronousMultiBlock::new().get_id(),
            NetCommandId::ClientSynchronousMultiBlock
        );
    }

    #[test]
    fn to_json_value_has_net_command_id_and_block_kind() {
        let t = BlockTarget::new("p1", BlockKind::STAB, None);
        let cmd = ClientCommandSynchronousMultiBlock::with_targets(vec![t]);
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientSynchronousMultiBlock");
        assert_eq!(json["selectedBlockTargets"][0]["blockKind"], "STAB");
    }

    #[test]
    fn round_trip_with_targets_and_entropy() {
        let t1 = BlockTarget::new("p1", BlockKind::BLOCK, None);
        let t2 = BlockTarget::new("p2", BlockKind::CHAINSAW, None);
        let mut cmd = ClientCommandSynchronousMultiBlock::with_targets(vec![t1, t2]);
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandSynchronousMultiBlock::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.selected_block_targets.len(), 2);
        assert_eq!(restored.selected_block_targets[0].get_player_id().map(|s| s.as_str()), Some("p1"));
        assert_eq!(restored.selected_block_targets[1].get_kind(), Some(BlockKind::CHAINSAW));
    }

    #[test]
    fn from_json_caps_at_two_targets() {
        let t1 = BlockTarget::new("p1", BlockKind::BLOCK, None);
        let t2 = BlockTarget::new("p2", BlockKind::STAB, None);
        let t3 = BlockTarget::new("p3", BlockKind::VOMIT, None);
        let cmd = ClientCommandSynchronousMultiBlock::with_targets(vec![t1, t2, t3]);
        let json = cmd.to_json_value();
        let restored = ClientCommandSynchronousMultiBlock::from_json(&json);
        assert_eq!(restored.selected_block_targets.len(), 2);
    }

    #[test]
    fn round_trip_with_no_targets() {
        let cmd = ClientCommandSynchronousMultiBlock::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandSynchronousMultiBlock::from_json(&json);
        assert!(restored.selected_block_targets.is_empty());
    }
}
