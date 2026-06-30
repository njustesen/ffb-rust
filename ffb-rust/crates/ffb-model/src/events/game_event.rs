use serde::{Deserialize, Serialize};
use crate::enums::{
    KickoffResult, PassResult, PassingDistance, PlayerAction,
    ReRollSource, SeriousInjuryKind, Weather,
};
use crate::model::player::PlayerId;
use crate::types::FieldCoordinate;

/// Every reportable game event (replaces 191+ Java XxxMessage classes).
///
/// The engine returns `Vec<GameEvent>` from every state-mutation call.
/// The parity runner serialises these to a canonical log for hash comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameEvent {
    // ── Rolls ──────────────────────────────────────────────────────────────────
    AlwaysHungry { player_id: PlayerId, roll: i32, success: bool },
    LonerRoll { player_id: PlayerId, roll: i32, success: bool },
    ProRoll { player_id: PlayerId, roll: i32, success: bool },
    TeamCaptainRoll { team_id: String, roll: i32, reroll_saved: bool },
    AnimosityRoll { player_id: PlayerId, roll: i32, success: bool },
    ArgueTheCall { player_id: PlayerId, roll: i32, success: bool },
    BloodLustRoll { player_id: PlayerId, roll: i32, success: bool },
    BribesRoll { player_id: PlayerId, roll: i32, success: bool },
    CatchRoll { player_id: PlayerId, target: i32, roll: i32, success: bool, rerolled: bool },
    PickupRoll { player_id: PlayerId, target: i32, roll: i32, success: bool, rerolled: bool },
    BalefulHexRoll { attacker_id: PlayerId, target_id: PlayerId, roll: i32, success: bool, rerolled: bool },
    ChainsawRoll { player_id: PlayerId, roll: i32 },
    LookIntoMyEyesRoll { player_id: PlayerId, roll: i32, success: bool, rerolled: bool },
    PickMeUpRoll { player_id: PlayerId, roll: i32, success: bool },
    ConfusionRoll { player_id: PlayerId, roll: i32, confused: bool },
    DauntlessRoll { player_id: PlayerId, roll: i32, success: bool },
    DodgeRoll { player_id: PlayerId, target: i32, roll: i32, success: bool, rerolled: bool },
    EscapeRoll { player_id: PlayerId, roll: i32, success: bool },
    FoulAppearanceRoll { player_id: PlayerId, roll: i32, failed: bool },
    GoForItRoll { player_id: PlayerId, target: i32, roll: i32, success: bool, rerolled: bool },
    HypnoticGazeRoll { player_id: PlayerId, target_id: PlayerId, roll: i32, success: bool },
    InterceptionRoll { player_id: PlayerId, target: i32, roll: i32, success: bool },
    JumpRoll { player_id: PlayerId, target: i32, roll: i32, success: bool },
    JumpUpRoll { player_id: PlayerId, target: i32, roll: i32, success: bool },
    MasterChefRoll { team_id: String, roll: i32, rerolls_stolen: i32 },
    PassRoll { player_id: PlayerId, target: i32, distance: PassingDistance, roll: i32, result: PassResult, rerolled: bool },
    PilingOn { player_id: PlayerId, target_id: PlayerId, rerolled: bool },
    PrayerRoll { team_id: String, roll: i32, prayer_id: String },
    RegenerationRoll { player_id: PlayerId, roll: i32, success: bool },
    RightStuffRoll { player_id: PlayerId, roll: i32, success: bool },
    SafeThrowRoll { player_id: PlayerId, roll: i32, success: bool },
    StandUpRoll { player_id: PlayerId, target: i32, roll: i32, success: bool },
    SwarmingPlayersRoll { team_id: String, roll: i32 },
    ThrowTeamMateRoll { thrower_id: PlayerId, thrown_id: PlayerId, roll: i32, result: PassResult },
    WeepingDaggerRoll { player_id: PlayerId, roll: i32 },
    SpellEffectRoll { roll: i32 },
    BreatheFireRoll { attacker_id: PlayerId, defender_id: PlayerId, roll: i32, knock_down: bool, prone: bool, failure: bool, rerolled: bool },
    ProjectileVomitRoll { attacker_id: PlayerId, defender_id: PlayerId, roll: i32, success: bool, rerolled: bool },
    TrapDoor { player_id: PlayerId, roll: i32, escaped: bool },

    // ── Block ──────────────────────────────────────────────────────────────────
    BlockRoll {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        nr_of_dice: i32,
        dice: Vec<i32>,
        selected_index: i32,
        own_choice: bool,
        rerolled: bool,
    },

    // ── Injury ─────────────────────────────────────────────────────────────────
    Injury {
        player_id: PlayerId,
        armor_roll: Option<[i32; 2]>,
        injury_roll: Option<[i32; 2]>,
        serious_injury: Option<SeriousInjuryKind>,
        was_ko: bool,
        was_cas: bool,
    },

    // ── Movement ───────────────────────────────────────────────────────────────
    PlayerAction { player_id: PlayerId, action: PlayerAction },
    PlayerMoved { player_id: PlayerId, coord: FieldCoordinate },
    PlayerFellDown { player_id: PlayerId, coord: FieldCoordinate },
    Pushback { attacker_id: PlayerId, defender_id: PlayerId, squares: Vec<FieldCoordinate> },
    ScatterBall { from: FieldCoordinate, directions: Vec<i32> },
    ScatterPlayer { player_id: PlayerId, coords: Vec<FieldCoordinate> },
    SwoopPlayer { player_id: PlayerId, coord: FieldCoordinate },
    ThrowIn { coord: FieldCoordinate, direction: i32, distance: i32 },

    // ── Ball handling ──────────────────────────────────────────────────────────
    BallPickedUp { player_id: PlayerId, coord: FieldCoordinate },
    BallScattered { player_id: PlayerId, coord: FieldCoordinate, direction: i32 },
    Touchdown { player_id: PlayerId, coord: FieldCoordinate },
    HandOver { from_id: PlayerId, to_id: PlayerId },
    KickoffScatter { start: FieldCoordinate, direction: i32, distance: i32 },
    PassDeviate { from: FieldCoordinate, scatter_directions: Vec<i32> },

    // ── Kickoff ────────────────────────────────────────────────────────────────
    KickoffResultEvent { result: KickoffResult },
    KickoffPitchInvasion { home_roll: i32, away_roll: i32 },
    KickoffPitchInvasionStun { player_id: PlayerId },
    KickoffRiot,
    KickoffThrowARock { player_id: Option<PlayerId> },
    RiotousRookies { team_id: String, player_count: i32 },

    // ── Re-rolls ───────────────────────────────────────────────────────────────
    ReRoll { team_id: String, source: ReRollSource, rerolled_action: String },

    // ── Skills ─────────────────────────────────────────────────────────────────
    SkillUse { player_id: PlayerId, skill_id: u16, used: bool },
    AnimalSavagery { player_id: PlayerId, roll: i32, success: bool },
    Leader { player_id: PlayerId, reroll_available: bool },
    ThenIStartedBlastin { player_id: PlayerId },

    // ── Inducements & cards ────────────────────────────────────────────────────
    ApothecaryChoice { player_id: PlayerId, healed: bool },
    CardDeactivated { card_id: String },
    CardEffectRoll { card_id: String, roll: i32, effect: String },
    DefectingPlayers { player_ids: Vec<PlayerId> },
    PassBlock { player_id: Option<PlayerId> },
    PassBlockEligible { player_ids: Vec<PlayerId>, home_defending: bool },
    PettyCash { team_id: String, amount: i32 },
    PlayCard { team_id: String, card_id: String },
    BuyInducement { team_id: String, inducement_id: String, count: u32 },
    SecretWeaponBan { player_id: PlayerId },
    PlayerEjected { player_id: PlayerId },
    CoachBanned { team_id: String },
    WizardUse { team_id: String, spell: String, coord: Option<FieldCoordinate> },
    BombExplodesAfterCatch { player_id: PlayerId, coord: FieldCoordinate },
    BombOutOfBounds { coord: FieldCoordinate },

    // ── Weather effects ────────────────────────────────────────────────────────
    HeatExhaustion { player_id: PlayerId },

    // ── Fouls ──────────────────────────────────────────────────────────────────
    /// Java: ReportReferee — whether the referee noticed the foul (and under-scrutiny flag).
    RefereeSpotsFoul { referee_spots_foul: bool, under_scrutiny: bool },
    /// Java: ReportBiasedRef — result of a biased-ref inducement roll.
    BiasedRefRoll { roll: i32, referee_spots_foul: bool },

    // ── Game flow ──────────────────────────────────────────────────────────────
    CoinThrow { home_won: bool },
    DoubleHiredStarPlayer,
    GameOptions { options_snapshot: std::collections::HashMap<String, String> },
    ReceiveChoice { team_id: String, receive: bool },
    StartHalf { half: i32 },
    TimeoutEnforced { team_id: String },
    TurnEnd { team_id: String, turn_nr: i32 },
    WeatherChange { weather: Weather },
    WinningsRoll { team_id: String, base: i32, roll: i32, total: i32 },
    MvpRoll { team_id: String, player_id: PlayerId, spp: i32 },
    Foul { attacker_id: PlayerId, defender_id: PlayerId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_coin_throw() {
        let ev = GameEvent::CoinThrow { home_won: true };
        let json = serde_json::to_string(&ev).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_dodge_roll() {
        let ev = GameEvent::DodgeRoll {
            player_id: "p1".into(),
            target: 3,
            roll: 4,
            success: true,
            rerolled: false,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_injury() {
        let ev = GameEvent::Injury {
            player_id: "p1".into(),
            armor_roll: Some([3, 4]),
            injury_roll: None,
            serious_injury: None,
            was_ko: false,
            was_cas: false,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
}
