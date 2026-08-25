use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;
use super::block_kind::BlockKind;

/// 1:1 translation of com.fumbbl.ffb.model.BlockTarget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockTarget {
    pub player_id: Option<PlayerId>,
    pub kind: Option<BlockKind>,
    pub original_player_state: Option<PlayerState>,
}

impl Default for BlockTarget {
    fn default() -> Self {
        BlockTarget { player_id: None, kind: None, original_player_state: None }
    }
}

/// A bare player id IS an ordinary BLOCK-kind target - that is exactly what the parameter used to
/// carry before it was widened to `BlockTarget`. STAB is always constructed explicitly, so this
/// conversion cannot silently produce the wrong kind.
impl From<&str> for BlockTarget {
    fn from(player_id: &str) -> Self { BlockTarget::block(player_id) }
}

impl From<String> for BlockTarget {
    fn from(player_id: String) -> Self { BlockTarget::block(player_id) }
}

impl BlockTarget {
    pub fn new(player_id: impl Into<PlayerId>, kind: BlockKind, original_player_state: Option<PlayerState>) -> Self {
        BlockTarget {
            player_id: Some(player_id.into()),
            kind: Some(kind),
            original_player_state,
        }
    }

    /// Convenience for the overwhelmingly common case: an ordinary BLOCK-kind target with no
    /// recorded original state. Java builds these client-side; the STAB kind is the rare one and
    /// is always constructed explicitly so it is visible at the call site.
    pub fn block(player_id: impl Into<PlayerId>) -> Self {
        BlockTarget::new(player_id, BlockKind::BLOCK, None)
    }

    pub fn get_player_id(&self) -> Option<&PlayerId> { self.player_id.as_ref() }
    pub fn get_kind(&self) -> Option<BlockKind> { self.kind }
    pub fn get_original_player_state(&self) -> Option<PlayerState> { self.original_player_state }

    /// Java: `BlockTarget.toJsonValue()`.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        if let Some(kind) = self.kind {
            map.insert("blockKind".to_string(), serde_json::json!(kind));
        }
        if let Some(player_id) = &self.player_id {
            map.insert("playerId".to_string(), serde_json::json!(player_id.as_str()));
        }
        if let Some(state) = self.original_player_state {
            map.insert("playerStateOld".to_string(), serde_json::json!(state));
        }
        serde_json::Value::Object(map)
    }

    /// Java: `BlockTarget.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let player_id = json.get("playerId").and_then(|v| v.as_str()).map(PlayerId::from);
        let kind = json
            .get("blockKind")
            .cloned()
            .and_then(|v| serde_json::from_value::<BlockKind>(v).ok());
        let original_player_state = json
            .get("playerStateOld")
            .cloned()
            .and_then(|v| serde_json::from_value::<PlayerState>(v).ok());
        BlockTarget { player_id, kind, original_player_state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of widening `StepParameter::BlockTargets` from `Vec<String>` to
    /// `Vec<BlockTarget>`: a target's `BlockKind` must survive the parameter. BB2020's
    /// multiple-block fork groups on exactly this to build its STAB bucket, which is what
    /// `ReportStabInjury` hangs off (docs/DEAD_STEP_INVENTORY.md).
    #[test]
    fn stab_kind_survives_where_a_bare_id_would_have_lost_it() {
        let ordinary: BlockTarget = "p1".into();
        assert_eq!(ordinary.get_kind(), Some(BlockKind::BLOCK),
            "a bare id must mean an ordinary BLOCK target");
        assert_eq!(ordinary.get_player_id().map(|p| p.as_str()), Some("p1"));

        let stab = BlockTarget::new("p2", BlockKind::STAB, None);
        assert_eq!(stab.get_kind(), Some(BlockKind::STAB));

        // Grouping by kind - the operation the fork performs - must separate the two.
        let targets = vec![ordinary, stab];
        let stabs: Vec<_> = targets.iter().filter(|t| t.get_kind() == Some(BlockKind::STAB)).collect();
        assert_eq!(stabs.len(), 1, "the STAB target must be distinguishable from the BLOCK one");
    }

    use super::*;
    #[test]
    fn serde_round_trip_default() {
        let t = BlockTarget::default();
        let s = serde_json::to_string(&t).unwrap();
        let back: BlockTarget = serde_json::from_str(&s).unwrap();
        assert!(back.player_id.is_none());
        assert!(back.kind.is_none());
    }

    #[test]
    fn to_json_value_has_block_kind_and_player_id() {
        let t = BlockTarget::new("p1", BlockKind::STAB, None);
        let json = t.to_json_value();
        assert_eq!(json["blockKind"], "STAB");
        assert_eq!(json["playerId"], "p1");
    }

    #[test]
    fn json_round_trip_with_all_fields() {
        let t = BlockTarget::new("p2", BlockKind::CHAINSAW, Some(PlayerState::new(4)));
        let json = t.to_json_value();
        let restored = BlockTarget::from_json(&json);
        assert_eq!(restored.get_player_id().map(|s| s.as_str()), Some("p2"));
        assert_eq!(restored.get_kind(), Some(BlockKind::CHAINSAW));
        assert_eq!(restored.get_original_player_state(), Some(PlayerState::new(4)));
    }

    #[test]
    fn json_round_trip_with_no_fields() {
        let t = BlockTarget::default();
        let json = t.to_json_value();
        let restored = BlockTarget::from_json(&json);
        assert!(restored.player_id.is_none());
        assert!(restored.kind.is_none());
        assert!(restored.original_player_state.is_none());
    }
}
