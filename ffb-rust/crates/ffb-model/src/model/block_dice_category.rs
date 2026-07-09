use crate::enums::DiceCategoryKind;

/// 1:1 translation of com.fumbbl.ffb.model.BlockDiceCategory.
pub struct BlockDiceCategory;

impl BlockDiceCategory {
    pub fn kind() -> DiceCategoryKind { DiceCategoryKind::Block }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_block() {
        assert_eq!(BlockDiceCategory::kind(), DiceCategoryKind::Block);
    }

    #[test]
    fn kind_is_not_direction() {
        assert_ne!(BlockDiceCategory::kind(), DiceCategoryKind::Direction);
    }
}
