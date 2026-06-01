use serde::{Deserialize, Serialize};

/// The five possible outcomes shown on a block die.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockResult {
    Skull,
    BothDown,
    Pushback,
    PowPushback,
    Pow,
}

impl BlockResult {
    pub fn name(self) -> &'static str {
        match self {
            BlockResult::Skull => "SKULL",
            BlockResult::BothDown => "BOTH DOWN",
            BlockResult::Pushback => "PUSHBACK",
            BlockResult::PowPushback => "POW/PUSH",
            BlockResult::Pow => "POW",
        }
    }

    pub fn from_name(name: &str) -> Option<BlockResult> {
        match name {
            "SKULL" => Some(BlockResult::Skull),
            "BOTH DOWN" => Some(BlockResult::BothDown),
            "PUSHBACK" => Some(BlockResult::Pushback),
            "POW/PUSH" => Some(BlockResult::PowPushback),
            "POW" => Some(BlockResult::Pow),
            _ => None,
        }
    }

    /// Die face value used in tests (1-indexed to match Java BlockEnums roll)
    pub fn die_value(self) -> u8 {
        match self {
            BlockResult::Skull => 1,
            BlockResult::BothDown => 2,
            BlockResult::Pushback => 3,
            BlockResult::PowPushback => 5,
            BlockResult::Pow => 6,
        }
    }

    pub fn all() -> &'static [BlockResult] {
        &[
            BlockResult::Skull,
            BlockResult::BothDown,
            BlockResult::Pushback,
            BlockResult::PowPushback,
            BlockResult::Pow,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_name() {
        for r in BlockResult::all() {
            assert_eq!(BlockResult::from_name(r.name()), Some(*r));
        }
    }

    #[test]
    fn serde_round_trip() {
        for r in BlockResult::all() {
            let json = serde_json::to_string(r).unwrap();
            let back: BlockResult = serde_json::from_str(&json).unwrap();
            assert_eq!(*r, back);
        }
    }

    #[test]
    fn block_result_count_is_five() {
        assert_eq!(BlockResult::all().len(), 5);
    }

    #[test]
    fn skull_name_is_skull() {
        assert_eq!(BlockResult::Skull.name(), "SKULL");
    }

    #[test]
    fn both_down_name_is_both_down() {
        assert_eq!(BlockResult::BothDown.name(), "BOTH DOWN");
    }

    #[test]
    fn pushback_name_is_pushback() {
        assert_eq!(BlockResult::Pushback.name(), "PUSHBACK");
    }

    #[test]
    fn pow_pushback_name_is_pow_push() {
        assert_eq!(BlockResult::PowPushback.name(), "POW/PUSH");
    }

    #[test]
    fn pow_name_is_pow() {
        assert_eq!(BlockResult::Pow.name(), "POW");
    }

    #[test]
    fn all_block_results_have_non_empty_names() {
        for r in BlockResult::all() {
            assert!(!r.name().is_empty());
        }
    }

    #[test]
    fn block_result_names_are_unique() {
        let names: Vec<&str> = BlockResult::all().iter().map(|r| r.name()).collect();
        let mut seen = std::collections::HashSet::new();
        for n in &names {
            assert!(seen.insert(*n), "duplicate name: {}", n);
        }
    }
}
