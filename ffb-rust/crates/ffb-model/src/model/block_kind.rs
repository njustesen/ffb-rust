use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.BlockKind.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    BLOCK,
    STAB,
    VOMIT,
    CHAINSAW,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(BlockKind::BLOCK, BlockKind::STAB);
        assert_ne!(BlockKind::VOMIT, BlockKind::CHAINSAW);
    }

    #[test]
    fn serde_round_trip() {
        let serialized = serde_json::to_string(&BlockKind::BLOCK).unwrap();
        let deserialized: BlockKind = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, BlockKind::BLOCK);
    }

    #[test]
    fn all_variants_serde_round_trip() {
        for v in [BlockKind::BLOCK, BlockKind::STAB, BlockKind::VOMIT, BlockKind::CHAINSAW] {
            let s = serde_json::to_string(&v).unwrap();
            let back: BlockKind = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn copy_semantics() {
        let a = BlockKind::CHAINSAW;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn equality_is_reflexive() {
        assert_eq!(BlockKind::STAB, BlockKind::STAB);
        assert_eq!(BlockKind::VOMIT, BlockKind::VOMIT);
    }
}
