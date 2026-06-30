use serde::{Deserialize, Serialize};

/// Kickoff table result — union of BB2016 / BB2020 / BB2025 variants.
/// Each edition only uses a subset; the engine dispatches on `Rules`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KickoffResult {
    // Common to all editions
    GetTheRef,
    HighKick,
    CheeringFans,
    WeatherChange,
    BrilliantCoaching,
    QuickSnap,
    Blitz,
    PitchInvasion,

    // BB2016-only
    Riot,
    PerfectDefence,
    ThrowARock,

    // BB2020/BB2025
    TimeOut,
    SolidDefence,
    OficiousRef,

    // BB2025-only
    Charge,
    DodgySnack,
}

impl KickoffResult {
    pub fn name(self) -> &'static str {
        match self {
            KickoffResult::GetTheRef => "Get the Ref",
            KickoffResult::HighKick => "High Kick",
            KickoffResult::CheeringFans => "Cheering Fans",
            KickoffResult::WeatherChange => "Weather Change",
            KickoffResult::BrilliantCoaching => "Brilliant Coaching",
            KickoffResult::QuickSnap => "Quick Snap",
            KickoffResult::Blitz => "Blitz",
            KickoffResult::PitchInvasion => "Pitch Invasion",
            KickoffResult::Riot => "Riot",
            KickoffResult::PerfectDefence => "Perfect Defence",
            KickoffResult::ThrowARock => "Throw a Rock",
            KickoffResult::TimeOut => "Time-out",
            KickoffResult::SolidDefence => "Solid Defence",
            KickoffResult::OficiousRef => "Officious Ref",
            KickoffResult::Charge => "Charge",
            KickoffResult::DodgySnack => "Dodgy Snack",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [
            KickoffResult::GetTheRef, KickoffResult::HighKick, KickoffResult::CheeringFans,
            KickoffResult::WeatherChange, KickoffResult::BrilliantCoaching, KickoffResult::QuickSnap,
            KickoffResult::Blitz, KickoffResult::PitchInvasion, KickoffResult::Riot,
            KickoffResult::PerfectDefence, KickoffResult::ThrowARock, KickoffResult::TimeOut,
            KickoffResult::SolidDefence, KickoffResult::OficiousRef, KickoffResult::Charge,
            KickoffResult::DodgySnack,
        ]
        .iter()
        .copied()
        .find(|v| v.name().eq_ignore_ascii_case(name))
    }

    pub fn is_fan_reroll(self) -> bool {
        self == KickoffResult::CheeringFans
    }

    pub fn is_coach_reroll(self) -> bool {
        self == KickoffResult::BrilliantCoaching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for k in &[
            KickoffResult::Blitz,
            KickoffResult::Riot,
            KickoffResult::Charge,
        ] {
            let json = serde_json::to_string(k).unwrap();
            let back: KickoffResult = serde_json::from_str(&json).unwrap();
            assert_eq!(*k, back);
        }
    }

    #[test]
    fn from_name_blitz() {
        assert_eq!(KickoffResult::from_name("Blitz"), Some(KickoffResult::Blitz));
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(KickoffResult::from_name("blitz"), Some(KickoffResult::Blitz));
    }

    #[test]
    fn from_name_unknown() {
        assert_eq!(KickoffResult::from_name("unknown"), None);
    }
}
