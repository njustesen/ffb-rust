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
