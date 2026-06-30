use serde::{Deserialize, Serialize};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;
use super::block_kind::BlockKind;

/// 1:1 translation of com.fumbbl.ffb.model.BlockTarget.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl BlockTarget {
    pub fn new(player_id: impl Into<PlayerId>, kind: BlockKind, original_player_state: Option<PlayerState>) -> Self {
        BlockTarget {
            player_id: Some(player_id.into()),
            kind: Some(kind),
            original_player_state,
        }
    }

    pub fn get_player_id(&self) -> Option<&PlayerId> { self.player_id.as_ref() }
    pub fn get_kind(&self) -> Option<BlockKind> { self.kind }
    pub fn get_original_player_state(&self) -> Option<PlayerState> { self.original_player_state }
}
