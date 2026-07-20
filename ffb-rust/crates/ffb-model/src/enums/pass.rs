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

    pub fn from_name(name: &str) -> Option<Self> {
        [
            PassingDistance::QuickPass, PassingDistance::ShortPass,
            PassingDistance::LongPass, PassingDistance::LongBomb, PassingDistance::PassToPartner,
        ]
        .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
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

    pub fn for_shortcut(c: char) -> Option<Self> {
        [
            PassingDistance::QuickPass, PassingDistance::ShortPass,
            PassingDistance::LongPass, PassingDistance::LongBomb, PassingDistance::PassToPartner,
        ]
        .iter().copied().find(|v| v.shortcut() == c)
    }
}

/// The overall outcome of a pass attempt, for event reporting/`StepParameter` payloads.
///
/// **Renamed from `PassResult` (Phase AD)** to remove a name collision with
/// `ffb_mechanics::pass_result::PassResult` (the 1:1 port of Java's single `mechanics.PassResult`
/// enum, used internally by the pass-evaluation mechanics) — this type has no Java enum
/// counterpart at all; it's a Rust-only reporting/event-payload type. It adds `Caught`/
/// `MissedCatch`, outcomes the mechanics-layer enum has no concept of, because it also has to
/// represent what happened to the ball after the throw, not just the throw roll itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassOutcome {
    Complete,
    Inaccurate,
    Fumble,
    WildlyInaccurate,
    Caught,
    MissedCatch,
}

impl PassOutcome {
    pub fn name(self) -> &'static str {
        match self {
            PassOutcome::Complete => "complete",
            PassOutcome::Inaccurate => "inaccurate",
            PassOutcome::Fumble => "fumble",
            PassOutcome::WildlyInaccurate => "wildlyInaccurate",
            PassOutcome::Caught => "caught",
            PassOutcome::MissedCatch => "missedCatch",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [
            PassOutcome::Complete, PassOutcome::Inaccurate, PassOutcome::Fumble,
            PassOutcome::WildlyInaccurate, PassOutcome::Caught, PassOutcome::MissedCatch,
        ]
        .iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn is_successful(self) -> bool {
        self == PassOutcome::Complete || self == PassOutcome::Caught
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_2016_all_distances() {
        assert_eq!(PassingDistance::QuickPass.modifier_2016(), 1);
        assert_eq!(PassingDistance::ShortPass.modifier_2016(), 0);
        assert_eq!(PassingDistance::LongPass.modifier_2016(), -1);
        assert_eq!(PassingDistance::LongBomb.modifier_2016(), -2);
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

    // ── EnumRoundTripTest parity (Java ffb-server) ──────────────────────────

    const ALL_DISTANCES: [PassingDistance; 5] = [
        PassingDistance::QuickPass,
        PassingDistance::ShortPass,
        PassingDistance::LongPass,
        PassingDistance::LongBomb,
        PassingDistance::PassToPartner,
    ];

    #[test]
    fn passing_distance_all_values_have_name() {
        for pd in ALL_DISTANCES {
            assert!(!pd.name().is_empty(), "All PassingDistance values must have a non-null name");
        }
    }

    #[test]
    fn passing_distance_bb2016_quick_pass_is_plus_1() {
        assert_eq!(PassingDistance::QuickPass.modifier_2016(), 1);
    }

    #[test]
    fn passing_distance_bb2016_short_pass_is_zero() {
        assert_eq!(PassingDistance::ShortPass.modifier_2016(), 0);
    }

    #[test]
    fn passing_distance_bb2016_long_pass_is_minus_1() {
        assert_eq!(PassingDistance::LongPass.modifier_2016(), -1);
    }

    #[test]
    fn passing_distance_bb2016_long_bomb_is_minus_2() {
        assert_eq!(PassingDistance::LongBomb.modifier_2016(), -2);
    }

    #[test]
    fn passing_distance_bb2020_quick_pass_is_zero() {
        assert_eq!(PassingDistance::QuickPass.modifier_2020(), 0);
    }

    #[test]
    fn passing_distance_bb2020_short_pass_is_1() {
        assert_eq!(PassingDistance::ShortPass.modifier_2020(), 1);
    }

    #[test]
    fn passing_distance_bb2020_long_pass_is_2() {
        assert_eq!(PassingDistance::LongPass.modifier_2020(), 2);
    }

    #[test]
    fn passing_distance_bb2020_long_bomb_is_3() {
        assert_eq!(PassingDistance::LongBomb.modifier_2020(), 3);
    }

    #[test]
    fn passing_distance_shortcuts_are_unique() {
        let shortcuts: std::collections::HashSet<char> =
            ALL_DISTANCES.iter().map(|pd| pd.shortcut()).collect();
        assert_eq!(
            shortcuts.len(),
            ALL_DISTANCES.len(),
            "All PassingDistance shortcuts must be unique"
        );
    }

    #[test]
    fn passing_distance_quick_pass_shortcut_is_q() {
        assert_eq!(PassingDistance::QuickPass.shortcut(), 'Q');
    }

    #[test]
    fn passing_distance_long_bomb_shortcut_is_b() {
        assert_eq!(PassingDistance::LongBomb.shortcut(), 'B');
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
