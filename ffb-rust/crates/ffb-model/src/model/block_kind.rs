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

}
