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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_1_is_skull() {
        assert_eq!(BlockResultCalc::block_result_for_roll(1), BlockResult::Skull);
    }

    #[test]
    fn roll_2_is_both_down() {
        assert_eq!(BlockResultCalc::block_result_for_roll(2), BlockResult::BothDown);
    }

    #[test]
    fn roll_3_is_pushback() {
        assert_eq!(BlockResultCalc::block_result_for_roll(3), BlockResult::Pushback);
    }

    #[test]
    fn roll_4_is_pushback() {
        assert_eq!(BlockResultCalc::block_result_for_roll(4), BlockResult::Pushback);
    }

    #[test]
    fn roll_5_is_pow_pushback() {
        assert_eq!(BlockResultCalc::block_result_for_roll(5), BlockResult::PowPushback);
    }

    #[test]
    fn roll_6_is_pow() {
        assert_eq!(BlockResultCalc::block_result_for_roll(6), BlockResult::Pow);
    }

    #[test]
    fn all_six_faces_map_correctly() {
        let expected = [
            BlockResult::Skull,
            BlockResult::BothDown,
            BlockResult::Pushback,
            BlockResult::Pushback,
            BlockResult::PowPushback,
            BlockResult::Pow,
        ];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(BlockResultCalc::block_result_for_roll(i as i32 + 1), *exp);
        }
    }
}
