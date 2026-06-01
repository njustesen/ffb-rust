use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::{BlockKind, PlayerState, ReRollSource};

/// State for a single block die roll, including re-roll tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockRoll {
    pub id: i32,
    pub target_id: String,
    pub old_player_state: PlayerState,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub selected_index: i32,
    pub own_choice: bool,
    pub successful_dauntless: bool,
    pub double_target_strength: bool,
    pub pro_index: i32,
    pub reroll_dice_indexes: Vec<i32>,
    pub reroll_sources: HashSet<ReRollSource>,
}

impl BlockRoll {
    pub fn new(target_id: String, old_player_state: PlayerState, id: i32) -> Self {
        BlockRoll {
            id,
            target_id,
            old_player_state,
            nr_of_dice: 0,
            block_roll: Vec::new(),
            selected_index: -1,
            own_choice: false,
            successful_dauntless: false,
            double_target_strength: false,
            pro_index: 0,
            reroll_dice_indexes: Vec::new(),
            reroll_sources: HashSet::new(),
        }
    }

    pub fn needs_selection(&self) -> bool {
        self.selected_index < 0
    }

    pub fn has_rerolls_left(&self) -> bool {
        !self.reroll_sources.is_empty()
    }

    pub fn index_was_rerolled(&self, index: i32) -> bool {
        self.reroll_dice_indexes.contains(&index)
    }
}

/// Extended block roll context carrying re-roll properties (used by the re-roll dialog).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockRollProperties {
    pub id: i32,
    pub target_id: String,
    pub old_player_state: PlayerState,
    pub nr_of_dice: i32,
    pub block_roll: Vec<i32>,
    pub selected_index: i32,
    pub own_choice: bool,
    pub successful_dauntless: bool,
    pub double_target_strength: bool,
    pub pro_index: i32,
    pub reroll_dice_indexes: Vec<i32>,
    /// Map of reroll-action-id → reroll-source-name (for multi-source rerolls).
    pub rr_action_to_source: HashMap<String, String>,
    /// Named properties (e.g. "canReRollBothDice").
    pub reroll_properties: HashSet<String>,
}

impl BlockRollProperties {
    pub fn new(target_id: String, old_player_state: PlayerState, id: i32) -> Self {
        BlockRollProperties {
            id,
            target_id,
            old_player_state,
            nr_of_dice: 0,
            block_roll: Vec::new(),
            selected_index: -1,
            own_choice: false,
            successful_dauntless: false,
            double_target_strength: false,
            pro_index: 0,
            reroll_dice_indexes: Vec::new(),
            rr_action_to_source: HashMap::new(),
            reroll_properties: HashSet::new(),
        }
    }

    pub fn needs_selection(&self) -> bool {
        self.selected_index < 0
    }

    pub fn has_rerolls_left(&self) -> bool {
        !self.rr_action_to_source.is_empty()
    }
}

/// Identifies a block target with the kind of block being made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockTarget {
    pub player_id: String,
    pub kind: BlockKind,
    pub original_player_state: PlayerState,
}

impl BlockTarget {
    pub fn new(player_id: String, kind: BlockKind, original_player_state: PlayerState) -> Self {
        BlockTarget { player_id, kind, original_player_state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{BlockKind, PlayerState};

    #[test]
    fn block_roll_needs_selection() {
        let mut br = BlockRoll::new("p1".into(), PlayerState(1), 1);
        assert!(br.needs_selection());
        br.selected_index = 0;
        assert!(!br.needs_selection());
    }

    #[test]
    fn block_roll_reroll_tracking() {
        let mut br = BlockRoll::new("p1".into(), PlayerState(1), 1);
        br.reroll_dice_indexes = vec![0, 2];
        assert!(br.index_was_rerolled(0));
        assert!(!br.index_was_rerolled(1));
        assert!(br.index_was_rerolled(2));
    }

    #[test]
    fn block_target_serde() {
        let bt = BlockTarget::new("p2".into(), BlockKind::Normal, PlayerState(0x00001));
        let json = serde_json::to_string(&bt).unwrap();
        let back: BlockTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(bt, back);
    }

    #[test]
    fn block_roll_has_rerolls_left_when_source_added() {
        use crate::enums::ReRollSource;
        let mut br = BlockRoll::new("p1".into(), PlayerState(1), 1);
        assert!(!br.has_rerolls_left());
        br.reroll_sources.insert(ReRollSource::new("trr"));
        assert!(br.has_rerolls_left());
    }

    #[test]
    fn block_roll_serde_round_trip() {
        let br = BlockRoll::new("p1".into(), PlayerState(1), 42);
        let json = serde_json::to_string(&br).unwrap();
        let back: BlockRoll = serde_json::from_str(&json).unwrap();
        assert_eq!(br, back);
    }
}
