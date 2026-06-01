use serde::{Deserialize, Serialize};

/// How far a pass travels — affects the target number modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassingDistance {
    QuickPass,
    ShortPass,
    LongPass,
    LongBomb,
    PassToPartner,
}

impl PassingDistance {
    pub fn name(self) -> &'static str {
        match self {
            PassingDistance::QuickPass => "Quick Pass",
            PassingDistance::ShortPass => "Short Pass",
            PassingDistance::LongPass => "Long Pass",
            PassingDistance::LongBomb => "Long Bomb",
            PassingDistance::PassToPartner => "Pass to Partner",
        }
    }

    pub fn modifier_2016(self) -> i32 {
        match self {
            PassingDistance::QuickPass => 1,
            PassingDistance::ShortPass => 0,
            PassingDistance::LongPass => -1,
            PassingDistance::LongBomb => -2,
            PassingDistance::PassToPartner => 0,
        }
    }

    pub fn modifier_2020(self) -> i32 {
        match self {
            PassingDistance::QuickPass => 0,
            PassingDistance::ShortPass => 1,
            PassingDistance::LongPass => 2,
            PassingDistance::LongBomb => 3,
            PassingDistance::PassToPartner => 0,
        }
    }

    pub fn shortcut(self) -> char {
        match self {
            PassingDistance::QuickPass => 'Q',
            PassingDistance::ShortPass => 'S',
            PassingDistance::LongPass => 'L',
            PassingDistance::LongBomb => 'B',
            PassingDistance::PassToPartner => 'R',
        }
    }
}

/// The overall outcome of a pass attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassResult {
    Complete,
    Inaccurate,
    Fumble,
    WildlyInaccurate,
    Caught,
    MissedCatch,
}

impl PassResult {
    pub fn name(self) -> &'static str {
        match self {
            PassResult::Complete => "complete",
            PassResult::Inaccurate => "inaccurate",
            PassResult::Fumble => "fumble",
            PassResult::WildlyInaccurate => "wildlyInaccurate",
            PassResult::Caught => "caught",
            PassResult::MissedCatch => "missedCatch",
        }
    }

    pub fn is_successful(self) -> bool {
        self == PassResult::Complete || self == PassResult::Caught
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_2016_quick_is_positive() {
        assert_eq!(PassingDistance::QuickPass.modifier_2016(), 1);
        assert_eq!(PassingDistance::LongBomb.modifier_2016(), -2);
    }

    #[test]
    fn modifier_2016_all_distances() {
        assert_eq!(PassingDistance::QuickPass.modifier_2016(), 1);
        assert_eq!(PassingDistance::ShortPass.modifier_2016(), 0);
        assert_eq!(PassingDistance::LongPass.modifier_2016(), -1);
        assert_eq!(PassingDistance::LongBomb.modifier_2016(), -2);
    }

    #[test]
    fn modifier_2020_is_penalty() {
        assert_eq!(PassingDistance::QuickPass.modifier_2020(), 0);
        assert_eq!(PassingDistance::LongBomb.modifier_2020(), 3);
    }

    #[test]
    fn modifier_2020_all_distances() {
        assert_eq!(PassingDistance::QuickPass.modifier_2020(), 0);
        assert_eq!(PassingDistance::ShortPass.modifier_2020(), 1);
        assert_eq!(PassingDistance::LongPass.modifier_2020(), 2);
        assert_eq!(PassingDistance::LongBomb.modifier_2020(), 3);
    }

    #[test]
    fn bb2016_vs_bb2020_quick_pass_differs() {
        // BB2016 quick pass gives +1 (bonus), BB2020 gives 0 (no modifier)
        assert_eq!(PassingDistance::QuickPass.modifier_2016(), 1);
        assert_eq!(PassingDistance::QuickPass.modifier_2020(), 0);
    }

    #[test]
    fn bb2016_vs_bb2020_long_bomb_differs() {
        // BB2016 long bomb is -2, BB2020 long bomb adds 3 (target number goes up)
        assert_eq!(PassingDistance::LongBomb.modifier_2016(), -2);
        assert_eq!(PassingDistance::LongBomb.modifier_2020(), 3);
    }

    #[test]
    fn pass_to_partner_bb2016_is_zero() {
        assert_eq!(PassingDistance::PassToPartner.modifier_2016(), 0);
    }

    #[test]
    fn pass_to_partner_bb2020_is_zero() {
        assert_eq!(PassingDistance::PassToPartner.modifier_2020(), 0);
    }

    #[test]
    fn serde_passing_distance() {
        let d = PassingDistance::LongBomb;
        let json = serde_json::to_string(&d).unwrap();
        let back: PassingDistance = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn serde_all_distances_round_trip() {
        for d in [
            PassingDistance::QuickPass,
            PassingDistance::ShortPass,
            PassingDistance::LongPass,
            PassingDistance::LongBomb,
            PassingDistance::PassToPartner,
        ] {
            let json = serde_json::to_string(&d).unwrap();
            let back: PassingDistance = serde_json::from_str(&json).unwrap();
            assert_eq!(d, back, "serde round-trip failed for {d:?}");
        }
    }
}
