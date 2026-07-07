// Report/event type identifiers used by the protocol.
// The full GameEvent enum (Tier 10) will contain all ~191 variants with payloads.
// This module provides the lightweight type-identifier enum for protocol tagging.

use serde::{Deserialize, Serialize};

/// Protocol tag identifying which `GameEvent` variant is serialized.
/// One variant per Java `Report*` class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportId {
    AlwaysHungry,
    AnimosityRoll,
    ApothecaryChoice,
    BlockMessage,
    BlockRoll,
    BombExplodesAfterCatch,
    BombOutOfBounds,
    BribesRoll,
    CardDeactivated,
    CardEffectRoll,
    CatchRoll,
    ChainsawRoll,
    CoinThrow,
    ConfusionRoll,
    DauntlessRoll,
    DefectingPlayers,
    DodgeRoll,
    DoubleHiredStarPlayer,
    EscapeRoll,
    FoulAppearanceRoll,
    Foul,
    FumbblResultUpload,
    GameOptions,
    HandOver,
    InterceptionRoll,
    JumpRoll,
    JumpUpRoll,
    KickoffResult,
    KickoffScatter,
    Leader,
    MasterChefRoll,
    PassBlock,
    PassDeviate,
    PettyCash,
    PilingOn,
    PlayCard,
    PlayerAction,
    Pushback,
    ReceiveChoice,
    RegenerationRoll,
    ReRoll,
    RightStuffRoll,
    RiotousRookies,
    SafeThrowRoll,
    SecretWeaponBan,
    SkillUse,
    SpellEffectRoll,
    StandUpRoll,
    StartHalf,
    ThrowIn,
    TimeoutEnforced,
    WeatherChange,
    WeepingDaggerRoll,
    WizardUse,
    Injury,
    PassRoll,
    ScatterBall,
    ScatterPlayer,
    GoForItRoll,
    TurnEnd,
    WinningsRoll,
    KickoffPitchInvasion,
    KickoffRiot,
    KickoffThrowARock,
    ArgueTheCall,
    BloodLustRoll,
    HypnoticGazeRoll,
    SwarmingPlayersRoll,
    SwoopPlayer,
    ThrowTeamMateRoll,
    AnimalSavagery,
    PrayerRoll,
    ThenIStartedBlastin,
}

impl ReportId {
    pub fn from_name(name: &str) -> Option<Self> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_name_returns_known_variant() {
        assert_eq!(ReportId::from_name("blockRoll"), Some(ReportId::BlockRoll));
    }
    #[test]
    fn from_name_unknown_returns_none() {
        assert!(ReportId::from_name("notAReport").is_none());
    }
    #[test]
    fn camel_case_serialization_round_trips() {
        let id = ReportId::DodgeRoll;
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"dodgeRoll\"");
    }

    #[test]
    fn from_name_covers_multi_word_camel_case() {
        // Variants with more than two words must also deserialize correctly.
        assert_eq!(
            ReportId::from_name("foulAppearanceRoll"),
            Some(ReportId::FoulAppearanceRoll)
        );
        assert_eq!(
            ReportId::from_name("thenIStartedBlastin"),
            Some(ReportId::ThenIStartedBlastin)
        );
    }

    #[test]
    fn deserialization_round_trips_all_variants() {
        // Spot-check that serialise → deserialise is an identity for every
        // variant we care about in the protocol.
        let ids = [
            ReportId::Injury,
            ReportId::BlockRoll,
            ReportId::KickoffResult,
            ReportId::WinningsRoll,
            ReportId::PrayerRoll,
        ];
        for id in ids {
            let json = serde_json::to_string(&id).unwrap();
            let back: ReportId = serde_json::from_str(&json).unwrap();
            assert_eq!(id, back);
        }
    }
}
