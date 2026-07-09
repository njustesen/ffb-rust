use crate::enums::DiceCategoryKind;

/// 1:1 translation of com.fumbbl.ffb.model.DiceCategoryFactory.
pub struct DiceCategoryFactory;

impl DiceCategoryFactory {
    pub fn for_kind(kind: DiceCategoryKind) -> Option<DiceCategoryKind> { Some(kind) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::DiceCategoryKind;

    #[test]
    fn for_kind_block_returns_block() {
        assert_eq!(DiceCategoryFactory::for_kind(DiceCategoryKind::Block), Some(DiceCategoryKind::Block));
    }

    #[test]
    fn for_kind_direction_returns_direction() {
        assert_eq!(DiceCategoryFactory::for_kind(DiceCategoryKind::Direction), Some(DiceCategoryKind::Direction));
    }
}
