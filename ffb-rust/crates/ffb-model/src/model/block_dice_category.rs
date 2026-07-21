use crate::enums::DiceCategoryKind;

/// 1:1 translation of com.fumbbl.ffb.model.BlockDiceCategory.
pub struct BlockDiceCategory;

impl BlockDiceCategory {
    pub fn kind() -> DiceCategoryKind { DiceCategoryKind::Block }
}
