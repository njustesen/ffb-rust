use crate::enums::BlockResult;

/// 1:1 translation of com.fumbbl.ffb.factory.BlockResultFactory.
pub struct BlockResultFactory;

impl Default for BlockResultFactory {
    fn default() -> Self { Self }
}

impl BlockResultFactory {
    pub fn for_name(&self, name: &str) -> Option<BlockResult> {
        BlockResult::from_name(name)
    }

    /// Maps a d6 block die face to a BlockResult.
    pub fn for_roll(&self, roll: i32) -> Option<BlockResult> {
        match roll {
            1 => Some(BlockResult::Skull),
            2 => Some(BlockResult::BothDown),
            5 => Some(BlockResult::PowPushback),
            6 => Some(BlockResult::Pow),
            _ => Some(BlockResult::Pushback),
        }
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_roll_skull() {
        let f = BlockResultFactory;
        assert_eq!(f.for_roll(1), Some(BlockResult::Skull));
    }

    #[test]
    fn for_roll_both_down() {
        assert_eq!(BlockResultFactory.for_roll(2), Some(BlockResult::BothDown));
    }

    #[test]
    fn for_roll_pushback_default() {
        assert_eq!(BlockResultFactory.for_roll(3), Some(BlockResult::Pushback));
        assert_eq!(BlockResultFactory.for_roll(4), Some(BlockResult::Pushback));
    }

    #[test]
    fn for_roll_pow_pushback() {
        assert_eq!(BlockResultFactory.for_roll(5), Some(BlockResult::PowPushback));
    }

    #[test]
    fn for_roll_pow() {
        assert_eq!(BlockResultFactory.for_roll(6), Some(BlockResult::Pow));
    }

    #[test]
    fn for_name_round_trip() {
        let f = BlockResultFactory;
        assert_eq!(f.for_name("SKULL"), Some(BlockResult::Skull));
        assert_eq!(f.for_name("NONEXISTENT"), None);
    }
}
