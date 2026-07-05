use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;
use super::block_kind::BlockKind;

/// 1:1 translation of com.fumbbl.ffb.DiceDecoration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceDecoration {
    pub coordinate: Option<FieldCoordinate>,
    pub nr_of_dice: i32,
    pub block_kind: Option<BlockKind>,
}

impl DiceDecoration {
    pub fn new(coordinate: FieldCoordinate, nr_of_dice: i32, block_kind: BlockKind) -> Self {
        DiceDecoration { coordinate: Some(coordinate), nr_of_dice, block_kind: Some(block_kind) }
    }

    pub fn get_coordinate(&self) -> Option<&FieldCoordinate> { self.coordinate.as_ref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_kind(&self) -> Option<BlockKind> { self.block_kind }

    pub fn transform(&self) -> Self {
        DiceDecoration {
            coordinate: self.coordinate.as_ref().map(|c| c.transform()),
            nr_of_dice: self.nr_of_dice,
            block_kind: self.block_kind,
        }
    }
}

impl Default for DiceDecoration {
    fn default() -> Self {
        DiceDecoration { coordinate: None, nr_of_dice: 0, block_kind: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;

    #[test]
    fn new_stores_all_fields() {
        let coord = FieldCoordinate::new(5, 7);
        let d = DiceDecoration::new(coord, 2, BlockKind::BLOCK);
        assert_eq!(d.get_coordinate(), Some(&coord));
        assert_eq!(d.get_nr_of_dice(), 2);
        assert_eq!(d.get_block_kind(), Some(BlockKind::BLOCK));
    }

    #[test]
    fn transform_preserves_dice_count_and_block_kind() {
        let d = DiceDecoration::new(FieldCoordinate::new(5, 3), 3, BlockKind::STAB);
        let t = d.transform();
        assert_eq!(t.get_nr_of_dice(), 3);
        assert_eq!(t.get_block_kind(), Some(BlockKind::STAB));
    }

    #[test]
    fn default_has_no_coordinate_or_block_kind() {
        let d = DiceDecoration::default();
        assert_eq!(d.get_coordinate(), None);
        assert_eq!(d.get_block_kind(), None);
        assert_eq!(d.get_nr_of_dice(), 0);
    }
}
