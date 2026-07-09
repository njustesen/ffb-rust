use serde::{Deserialize, Serialize};
use crate::enums::Rules;

pub mod kickoff_result;
pub mod kickoff_result_mapping;

pub mod bb2016 {
    pub mod kickoff_result;
    pub mod kickoff_result_mapping;
}

pub mod bb2020 {
    pub mod kickoff_result;
    pub mod kickoff_result_mapping;
}

pub mod bb2025 {
    pub mod kickoff_result;
    pub mod kickoff_result_mapping;
}

/// All kickoff event variants across all editions.
///
/// Some variants only exist in specific editions (noted in doc comments).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KickoffEventKind {
    // ── Shared across all editions ─────────────────────────────────────────
    GetTheRef,
    HighKick,
    CheeringFans,
    WeatherChange,
    BrilliantCoaching,
    QuickSnap,
    PitchInvasion,

    // ── BB2016 only ────────────────────────────────────────────────────────
    Riot,
    PerfectDefence,
    ThrowARock,

    // ── BB2020 only ────────────────────────────────────────────────────────
    TimeOut,
    SolidDefence,
    OficiousRef,
    Blitz,

    // ── BB2025 only ────────────────────────────────────────────────────────
    /// BB2025 equivalent of BB2020 Blitz (renamed)
    Charge,
    DodgySnack,
}

impl KickoffEventKind {
    pub fn name(self) -> &'static str {
        match self {
            KickoffEventKind::GetTheRef => "Get the Ref",
            KickoffEventKind::HighKick => "High Kick",
            KickoffEventKind::CheeringFans => "Cheering Fans",
            KickoffEventKind::WeatherChange => "Weather Change",
            KickoffEventKind::BrilliantCoaching => "Brilliant Coaching",
            KickoffEventKind::QuickSnap => "Quick Snap",
            KickoffEventKind::PitchInvasion => "Pitch Invasion",
            KickoffEventKind::Riot => "Riot",
            KickoffEventKind::PerfectDefence => "Perfect Defence",
            KickoffEventKind::ThrowARock => "Throw a Rock",
            KickoffEventKind::TimeOut => "Time-out",
            KickoffEventKind::SolidDefence => "Solid Defence",
            KickoffEventKind::OficiousRef => "Officious Ref",
            KickoffEventKind::Blitz => "Blitz",
            KickoffEventKind::Charge => "Charge",
            KickoffEventKind::DodgySnack => "Dodgy Snack",
        }
    }

    /// Whether this result grants the team with more fans a re-roll (BB2016 Cheering Fans).
    pub fn is_fan_reroll(self) -> bool {
        matches!(self, KickoffEventKind::CheeringFans)
    }

    /// Whether this result grants the team with better coaching a re-roll.
    pub fn is_coach_reroll(self) -> bool {
        matches!(self, KickoffEventKind::BrilliantCoaching)
    }
}

/// Map a 2d6 roll to the kickoff event for a given rules edition.
///
/// Returns `None` for invalid rolls (< 2 or > 12) or rolls not in the edition's table.
pub fn kickoff_event(rules: Rules, roll: i32) -> Option<KickoffEventKind> {
    match rules {
        Rules::Bb2016 => kickoff_event_bb2016(roll),
        Rules::Bb2020 => kickoff_event_bb2020(roll),
        Rules::Bb2025 | Rules::Common => kickoff_event_bb2025(roll),
    }
}

/// BB2016 kickoff table (2d6).
pub fn kickoff_event_bb2016(roll: i32) -> Option<KickoffEventKind> {
    Some(match roll {
        2 => KickoffEventKind::GetTheRef,
        3 => KickoffEventKind::Riot,
        4 => KickoffEventKind::PerfectDefence,
        5 => KickoffEventKind::HighKick,
        6 => KickoffEventKind::CheeringFans,
        7 => KickoffEventKind::WeatherChange,
        8 => KickoffEventKind::BrilliantCoaching,
        9 => KickoffEventKind::QuickSnap,
        10 => KickoffEventKind::Blitz,
        11 => KickoffEventKind::ThrowARock,
        12 => KickoffEventKind::PitchInvasion,
        _ => return None,
    })
}

/// BB2020 kickoff table (2d6).
pub fn kickoff_event_bb2020(roll: i32) -> Option<KickoffEventKind> {
    Some(match roll {
        2 => KickoffEventKind::GetTheRef,
        3 => KickoffEventKind::TimeOut,
        4 => KickoffEventKind::SolidDefence,
        5 => KickoffEventKind::HighKick,
        6 => KickoffEventKind::CheeringFans,
        7 => KickoffEventKind::BrilliantCoaching,
        8 => KickoffEventKind::WeatherChange,
        9 => KickoffEventKind::QuickSnap,
        10 => KickoffEventKind::Blitz,
        11 => KickoffEventKind::OficiousRef,
        12 => KickoffEventKind::PitchInvasion,
        _ => return None,
    })
}

/// BB2025 kickoff table (2d6).
pub fn kickoff_event_bb2025(roll: i32) -> Option<KickoffEventKind> {
    Some(match roll {
        2 => KickoffEventKind::GetTheRef,
        3 => KickoffEventKind::TimeOut,
        4 => KickoffEventKind::SolidDefence,
        5 => KickoffEventKind::HighKick,
        6 => KickoffEventKind::CheeringFans,
        7 => KickoffEventKind::BrilliantCoaching,
        8 => KickoffEventKind::WeatherChange,
        9 => KickoffEventKind::QuickSnap,
        10 => KickoffEventKind::Charge,
        11 => KickoffEventKind::DodgySnack,
        12 => KickoffEventKind::PitchInvasion,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bb2020_full_table() {
        assert_eq!(kickoff_event(Rules::Bb2020, 2), Some(KickoffEventKind::GetTheRef));
        assert_eq!(kickoff_event(Rules::Bb2020, 7), Some(KickoffEventKind::BrilliantCoaching));
        assert_eq!(kickoff_event(Rules::Bb2020, 12), Some(KickoffEventKind::PitchInvasion));
    }

    #[test]
    fn bb2016_riot_on_3() {
        assert_eq!(kickoff_event(Rules::Bb2016, 3), Some(KickoffEventKind::Riot));
    }

    #[test]
    fn bb2025_charge_on_10() {
        assert_eq!(kickoff_event(Rules::Bb2025, 10), Some(KickoffEventKind::Charge));
    }

    #[test]
    fn invalid_roll_returns_none() {
        assert_eq!(kickoff_event(Rules::Bb2020, 1), None);
        assert_eq!(kickoff_event(Rules::Bb2020, 13), None);
    }

    #[test]
    fn bb2025_brilliant_coaching_on_7() {
        assert_eq!(kickoff_event(Rules::Bb2025, 7), Some(KickoffEventKind::BrilliantCoaching));
        assert_eq!(kickoff_event(Rules::Bb2025, 8), Some(KickoffEventKind::WeatherChange));
    }

    #[test]
    fn bb2025_full_table() {
        let expected: Vec<(i32, KickoffEventKind)> = vec![
            (2, KickoffEventKind::GetTheRef),
            (3, KickoffEventKind::TimeOut),
            (4, KickoffEventKind::SolidDefence),
            (5, KickoffEventKind::HighKick),
            (6, KickoffEventKind::CheeringFans),
            (7, KickoffEventKind::BrilliantCoaching),
            (8, KickoffEventKind::WeatherChange),
            (9, KickoffEventKind::QuickSnap),
            (10, KickoffEventKind::Charge),
            (11, KickoffEventKind::DodgySnack),
            (12, KickoffEventKind::PitchInvasion),
        ];
        for (roll, kind) in expected {
            assert_eq!(kickoff_event(Rules::Bb2025, roll), Some(kind), "roll {}", roll);
        }
    }

    #[test]
    fn brilliant_coaching_is_coach_reroll() {
        assert!(KickoffEventKind::BrilliantCoaching.is_coach_reroll());
        assert!(!KickoffEventKind::Blitz.is_coach_reroll());
    }

    #[test]
    fn cheering_fans_is_fan_reroll() {
        assert!(KickoffEventKind::CheeringFans.is_fan_reroll());
    }
}
