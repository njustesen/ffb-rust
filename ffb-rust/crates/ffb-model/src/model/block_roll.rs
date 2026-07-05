use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::model::player::PlayerId;
use crate::enums::PlayerState;

/// 1:1 translation of com.fumbbl.ffb.model.BlockRoll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRoll {
    pub target_id: Option<PlayerId>,
    pub old_player_state: Option<PlayerState>,
    pub successful_dauntless: bool,
    pub own_choice: bool,
    pub double_target_strength: bool,
    pub nr_of_dice: i32,
    pub id: i32,
    pub pro_index: i32,
    pub block_roll: Vec<i32>,
    pub re_roll_dice_indexes: Vec<i32>,
    pub selected_index: i32,
    pub re_roll_sources: HashSet<ReRollSource>,
}

impl Default for BlockRoll {
    fn default() -> Self {
        BlockRoll {
            target_id: None,
            old_player_state: None,
            successful_dauntless: false,
            own_choice: false,
            double_target_strength: false,
            nr_of_dice: 0,
            id: 0,
            pro_index: 0,
            block_roll: Vec::new(),
            re_roll_dice_indexes: Vec::new(),
            selected_index: -1,
            re_roll_sources: HashSet::new(),
        }
    }
}

impl BlockRoll {
    pub fn new_with(target_id: impl Into<PlayerId>, old_player_state: Option<PlayerState>, id: i32) -> Self {
        BlockRoll { target_id: Some(target_id.into()), old_player_state, id, ..Default::default() }
    }

    pub fn get_id(&self) -> i32 { self.id }
    pub fn get_target_id(&self) -> Option<&PlayerId> { self.target_id.as_ref() }
    pub fn is_successful_dauntless(&self) -> bool { self.successful_dauntless }
    pub fn set_successful_dauntless(&mut self, v: bool) { self.successful_dauntless = v; }
    pub fn is_double_target_strength(&self) -> bool { self.double_target_strength }
    pub fn set_double_target_strength(&mut self, v: bool) { self.double_target_strength = v; }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn set_nr_of_dice(&mut self, v: i32) { self.nr_of_dice = v; }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn set_block_roll(&mut self, roll: Vec<i32>) { self.block_roll = roll; }
    pub fn get_selected_index(&self) -> i32 { self.selected_index }
    pub fn set_selected_index(&mut self, v: i32) { self.selected_index = v; }
    pub fn is_own_choice(&self) -> bool { self.own_choice }
    pub fn set_own_choice(&mut self, v: bool) { self.own_choice = v; }
    pub fn get_pro_index(&self) -> i32 { self.pro_index }
    pub fn set_pro_index(&mut self, v: i32) { self.pro_index = v; }
    pub fn get_re_roll_dice_indexes(&self) -> &[i32] { &self.re_roll_dice_indexes }
    pub fn set_re_roll_dice_indexes(&mut self, v: Vec<i32>) { self.re_roll_dice_indexes = v; }
    pub fn index_was_re_rolled(&self, index: i32) -> bool {
        self.re_roll_dice_indexes.contains(&index)
    }
    pub fn add_re_roll_source(&mut self, source: ReRollSource) { self.re_roll_sources.insert(source); }
    pub fn remove_re_roll_source(&mut self, source: &ReRollSource) { self.re_roll_sources.remove(source); }
    pub fn clear_re_roll_sources(&mut self) { self.re_roll_sources.clear(); }
    pub fn has_re_roll_source(&self, source: &ReRollSource) -> bool { self.re_roll_sources.contains(source) }
    pub fn has_re_rolls_left(&self) -> bool { !self.re_roll_sources.is_empty() }
    /// Java: needsSelection() — true when no die has been chosen yet.
    pub fn needs_selection(&self) -> bool { self.selected_index < 0 }
    pub fn get_old_player_state(&self) -> Option<PlayerState> { self.old_player_state }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selected_index_is_minus_one() {
        assert_eq!(BlockRoll::default().get_selected_index(), -1);
    }

    #[test]
    fn index_was_re_rolled() {
        let mut br = BlockRoll::default();
        br.set_re_roll_dice_indexes(vec![0, 2]);
        assert!(br.index_was_re_rolled(0));
        assert!(!br.index_was_re_rolled(1));
        assert!(br.index_was_re_rolled(2));
    }

    #[test]
    fn re_roll_sources_initially_empty() {
        assert!(!BlockRoll::default().has_re_rolls_left());
    }

    #[test]
    fn serde_round_trip() {
        let mut br = BlockRoll::new_with("player1", None, 42);
        br.set_nr_of_dice(2);
        br.set_block_roll(vec![3, 5]);
        let json = serde_json::to_string(&br).unwrap();
        let back: BlockRoll = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), 42);
        assert_eq!(back.get_nr_of_dice(), 2);
        assert_eq!(back.get_block_roll(), &[3, 5]);
    }
}
