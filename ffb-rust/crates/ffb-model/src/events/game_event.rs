use serde::{Deserialize, Serialize};
use crate::enums::{
    Direction, KickoffResult, PassOutcome, PassingDistance, PlayerAction,
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
    ChainsawRoll { player_id: PlayerId, roll: i32, minimum_roll: i32, success: bool, rerolled: bool },
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
    PassRoll { player_id: PlayerId, target: i32, distance: PassingDistance, roll: i32, result: PassOutcome, rerolled: bool },
    PilingOn { player_id: PlayerId, target_id: PlayerId, rerolled: bool },
    PrayerRoll { team_id: String, roll: i32, prayer_id: String },
    RegenerationRoll { player_id: PlayerId, roll: i32, success: bool },
    RightStuffRoll { player_id: PlayerId, roll: i32, success: bool },
    SafeThrowRoll { player_id: PlayerId, roll: i32, success: bool },
    StandUpRoll { player_id: PlayerId, target: i32, roll: i32, success: bool },
    SwarmingPlayersRoll { team_id: String, roll: i32 },
    ThrowTeamMateRoll { thrower_id: PlayerId, thrown_id: PlayerId, roll: i32, result: PassOutcome },
    WeepingDaggerRoll { player_id: PlayerId, roll: i32 },
    SpellEffectRoll { roll: i32 },
    BreatheFireRoll { attacker_id: PlayerId, defender_id: PlayerId, roll: i32, knock_down: bool, prone: bool, failure: bool, rerolled: bool },
    ProjectileVomitRoll { attacker_id: PlayerId, defender_id: PlayerId, roll: i32, success: bool, rerolled: bool },
    TrapDoor { player_id: PlayerId, roll: i32, escaped: bool },

    // ── Block ──────────────────────────────────────────────────────────────────
    /// Java: ReportBlock — signals the start of a block action against a defender.
    Block { defender_id: PlayerId },
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
    /// Java: ReportKickoffRiot — riot result; roll is 0 when modifier was fixed by turn number.
    KickoffRiot { turn_modifier: i32, roll: i32 },
    KickoffThrowARock { player_id: Option<PlayerId> },
    RiotousRookies { team_id: String, player_count: i32 },

    // ── Re-rolls ───────────────────────────────────────────────────────────────
    ReRoll { team_id: String, source: ReRollSource, rerolled_action: String },

    // ── Skills ─────────────────────────────────────────────────────────────────
    SkillUse { player_id: PlayerId, skill_id: u16, used: bool },
    /// Java: ReportHitAndRun — attacker moves one square after block.
    HitAndRun { player_id: PlayerId, direction: Direction },
    /// Java: ReportKickTeamMateFumble — the kicked team-mate fumbled the throw.
    KickTeamMateFumble,
    /// Java: ReportPrayerAmount — TV difference prayer allocation report.
    PrayerAmount { tv_home: i32, tv_away: i32, prayer_amount: i32, home_team_receives_prayers: bool },
    /// Java: ReportBiteSpectator — Undead player bites a spectator (Blood Lust feeding on crowd).
    BiteSpectator { player_id: PlayerId },
    /// Java: ReportCardsAndInducementsBought — summary of what a team bought before kickoff.
    CardsAndInducementsBought { team_id: String, cards: i32, inducements: i32, stars: i32, mercenaries: i32, gold: i32, new_tv: i32 },
    /// Java: ReportKickoffSequenceActivationsExhausted — no more activations in kickoff sequence.
    KickoffSequenceActivationsExhausted { limit_reached: bool },
    /// Java: ReportSolidDefenceRoll — D3+3 roll for Solid Defence kickoff result.
    SolidDefenceRoll { team_id: String, roll: i32, amount: i32 },
    /// Java: ReportCheeringFans — fan factor + reroll adjustment from Cheering Fans kickoff.
    CheeringFans { home_roll: i32, away_roll: i32 },
    /// Java: ReportKickoffExtraReRoll — team gains an extra re-roll from Brilliant Coaching.
    KickoffExtraReRoll { team_id: String },
    /// Java: ReportQuickSnapRoll — D3+3 roll for Quick Snap kickoff result.
    QuickSnapRoll { team_id: String, roll: i32, amount: i32 },
    /// Java: ReportKickoffTimeout — turn counter shifted due to Timeout kickoff result.
    KickoffTimeout { turn_number: i32, turn_modifier: i32 },
    /// Java: ReportKickoffOfficiousRef — officious ref targets players; roll_home/away are the D6 rolls.
    KickoffOfficiousRef { roll_home: i32, roll_away: i32, player_ids: Vec<PlayerId> },
    /// Java: ReportKickoffDodgySnack — dodgy snack targets players; roll_home/away are the D6 rolls.
    KickoffDodgySnack { roll_home: i32, roll_away: i32, player_ids: Vec<PlayerId> },
    /// Java: ReportDodgySnackRoll — per-player D6 roll for the Dodgy Snack kickoff result.
    DodgySnackRoll { player_id: PlayerId, roll: i32 },
    /// Java: ReportThrowAtPlayer (BB2025) — rock thrown at player in Throw-a-Rock kickoff result.
    ThrowAtPlayer { player_id: PlayerId, roll: i32, successful: bool },
    /// Java: ReportFumblerooskie — fumblerooskie skill use report (BB2020/BB2025).
    Fumblerooskie { player_id: PlayerId, used: bool },
    /// Java: ReportAllYouCanEatRoll — All You Can Eat skill roll result (BB2020/BB2025).
    AllYouCanEatRoll { player_id: PlayerId, roll: i32, minimum_roll: i32, success: bool, rerolled: bool },
    /// Java: ReportKickoffExtraReRoll (BB2016) — combined extra reroll for CheeringFans/BrilliantCoaching.
    KickoffExtraReRollBb2016 { kickoff_result: KickoffResult, roll_home: i32, home_gains_reroll: bool, roll_away: i32, away_gains_reroll: bool },
    /// Java: ReportKickoffThrowARock (BB2016) — fans throw rocks at players; multi-player variant.
    KickoffThrowARockBb2016 { roll_home: i32, roll_away: i32, player_ids: Vec<PlayerId> },
    /// Java: ReportKickoffPitchInvasion (BB2016) — per-player d6 rolls for pitch invasion.
    KickoffPitchInvasionBb2016 { rolls_home: Vec<i32>, affected_home: Vec<bool>, rolls_away: Vec<i32>, affected_away: Vec<bool> },
    AnimalSavagery { player_id: PlayerId, roll: i32, success: bool },
    Leader { player_id: PlayerId, reroll_available: bool },
    ThenIStartedBlastin { attacker_id: PlayerId, defender_id: Option<PlayerId>, roll: i32, success: bool, fumble: bool },

    // ── Inducements & cards ────────────────────────────────────────────────────
    /// Java: ServerCommandAddPlayer — new player added to game (star player, mercenary, staff).
    /// Emitted once per added player from inducement steps.
    PlayerAdded { team_id: String, player_id: String, position_id: String },
    /// Java: ReportApothecaryRoll — apo re-rolls the casualty die. roll/new_state are None when not used.
    ApothecaryRoll { player_id: PlayerId, roll: Option<i32>, new_state: Option<u16>, new_serious_injury: Option<SeriousInjuryKind> },
    ApothecaryChoice { player_id: PlayerId, healed: bool },
    /// Java: ReportSelectBlitzTarget (BB2020/BB2025) — attacker selects a blitz target.
    SelectBlitzTarget { attacker_id: PlayerId, defender_id: PlayerId },
    /// Java: ReportSelectGazeTarget (BB2020/BB2025) — attacker selects a gaze target.
    SelectGazeTarget { attacker_id: PlayerId, defender_id: PlayerId },
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
    /// Java: ReportThrownKeg — BeerBarrelBash keg throw result.
    KegThrow { thrower_id: PlayerId, target_id: Option<PlayerId>, roll: i32, success: bool, fumble: bool },
    /// Java: ReportInducement — inducement registered at start of half (wandering apo, extra training, etc.).
    Inducement { team_id: String, inducement_type: String, value: i32 },
    /// Java: ReportPumpUpTheCrowdReRoll — attacker's PumpUpTheCrowd skill granted a re-roll.
    PumpUpTheCrowdReRoll { player_id: PlayerId },

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
    /// Java: ReportBlitzRoll — blitz activation roll result.
    BlitzRoll { team_id: String, roll: i32, limit: i32 },
    /// Java: ReportSpecialEffectRoll — special effect (ZAP, etc.) roll result.
    SpecialEffectRoll { effect: String, player_id: PlayerId, roll: i32, success: bool },
    /// Java: ReportThrowAtStallingPlayer — keg/ball thrown at stalling player.
    ThrowAtStallingPlayer { player_id: PlayerId, roll: i32, success: bool },
    /// Java: ReportNoPlayersToField — no players available to field for a team.
    NoPlayersToField { team_id: Option<String> },
    /// Java: ReportPlayerEvent — a notable player event (e.g. "is stalling", "awarded touchdown").
    PlayerNote { player_id: PlayerId, note: String },
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

    #[test]
    fn serde_block_roll_round_trips() {
        let ev = GameEvent::BlockRoll {
            attacker_id: "a1".into(),
            defender_id: "d1".into(),
            nr_of_dice: 2,
            dice: vec![3, 5],
            selected_index: 1,
            own_choice: true,
            rerolled: false,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_kickoff_throw_a_rock_none_player() {
        // KickoffThrowARock with no targeted player must round-trip with a null player_id.
        let ev = GameEvent::KickoffThrowARock { player_id: None };
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("null"), "expected null player_id in JSON: {}", json);
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
    #[test]
    fn debug_format_nonempty() {
        let ev = GameEvent::AlwaysHungry { player_id: "p1".to_string(), roll: 1, success: false };
        assert!(!format!("{:?}", ev).is_empty());
    }

}
