// 1:1 translation of com.fumbbl.ffb.server.util.BlockResultCalc
use ffb_model::enums::BlockResult;

pub struct BlockResultCalc;

impl BlockResultCalc {
    pub fn new() -> Self {
        Self
    }

    /// Maps a block die roll (1–6) to a BlockResult.
    /// 1 → Skull, 2 → BothDown, 3/4 → Pushback, 5 → PowPushback, 6 → Pow
    pub fn block_result_for_roll(roll: i32) -> BlockResult {
        match roll {
            1 => BlockResult::Skull,
            2 => BlockResult::BothDown,
            5 => BlockResult::PowPushback,
            6 => BlockResult::Pow,
            _ => BlockResult::Pushback,
        }
    }
}

impl Default for BlockResultCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Test mirror of com.fumbbl.ffb.server.util.BlockResultCalcTest
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_result_for_roll() {
        for (roll, expected) in [
            (1, BlockResult::Skull),
            (2, BlockResult::BothDown),
            (3, BlockResult::Pushback),
            (4, BlockResult::Pushback),
            (5, BlockResult::PowPushback),
            (6, BlockResult::Pow),
        ] {
            assert_eq!(BlockResultCalc::block_result_for_roll(roll), expected, "roll={roll}");
        }
    }

    #[test]
    fn skull_on_1() {
        assert_eq!(BlockResultCalc::block_result_for_roll(1), BlockResult::Skull);
    }

    #[test]
    fn both_down_on_2() {
        assert_eq!(BlockResultCalc::block_result_for_roll(2), BlockResult::BothDown);
    }

    #[test]
    fn pushback_on_3() {
        assert_eq!(BlockResultCalc::block_result_for_roll(3), BlockResult::Pushback);
    }

    #[test]
    fn pushback_on_4() {
        assert_eq!(BlockResultCalc::block_result_for_roll(4), BlockResult::Pushback);
    }

    #[test]
    fn pow_pushback_on_5() {
        assert_eq!(BlockResultCalc::block_result_for_roll(5), BlockResult::PowPushback);
    }

    #[test]
    fn pow_on_6() {
        assert_eq!(BlockResultCalc::block_result_for_roll(6), BlockResult::Pow);
    }
}
