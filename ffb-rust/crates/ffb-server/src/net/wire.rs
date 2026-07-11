/// Outgoing wire-format structs matching the Java FFB server's JSON output.
///
/// The Java client parses these by exact field name.  Field names here
/// intentionally preserve Java's camelCase JSON keys (e.g. `"reportId"`,
/// `"modelChangeList"`).  These are NOT the same as the `ffb_protocol`
/// `ServerCommand` structs, which model the client→server direction.
///
/// This file covers `GameEvent -> WireReport` (state-change log entries sent
/// to the client). The complementary direction — the engine's dialog/choice
/// request (`AgentPrompt`) turned into an outgoing wire dialog command — is
/// covered by the sibling module `wire_prompt.rs` (`AgentPrompt -> WireDialog`
/// via `prompt_to_wire`).
///
/// Reference: `ffb-common/.../net/commands/ServerCommandModelSync.java`
use serde::Serialize;
use ffb_model::events::GameEvent;

// ── ReportList / WireReport ────────────────────────────────────────────────

/// Java: `ReportList` → JSON `{ "reports": [...] }`
#[derive(Serialize)]
pub struct WireReportList {
    pub reports: Vec<WireReport>,
}

/// Java: `IReport` implementations → tagged by `"reportId"` field.
///
/// Each variant's `rename` must match the Java `ReportId` enum constant name
/// exactly (SCREAMING_SNAKE_CASE), e.g. `"BLOCK"`, `"DODGE_ROLL"`.
#[derive(Serialize)]
#[serde(tag = "reportId")]
pub enum WireReport {
    /// Java: `ReportBlock` — announces a block declaration.
    #[serde(rename = "BLOCK")]
    Block {
        #[serde(rename = "defenderId")]
        defender_id: String,
    },

    /// Java: `ReportBlockRoll` — announces block dice result.
    #[serde(rename = "BLOCK_ROLL")]
    BlockRoll {
        #[serde(rename = "attackerId")]
        attacker_id: String,
        #[serde(rename = "defenderId")]
        defender_id: String,
        #[serde(rename = "nrOfDice")]
        nr_of_dice: i32,
        dice: Vec<i32>,
        #[serde(rename = "selectedDie")]
        selected_die: i32,
        #[serde(rename = "ownChoice")]
        own_choice: bool,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportDodgeRoll` / `ReportSkillRoll` subclass.
    #[serde(rename = "DODGE_ROLL")]
    DodgeRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportGoForItRoll`
    #[serde(rename = "GO_FOR_IT_ROLL")]
    GoForItRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportPickupRoll`
    #[serde(rename = "PICKUP_ROLL")]
    PickupRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportCatchRoll`
    #[serde(rename = "CATCH_ROLL")]
    CatchRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
    },

    /// Java: `ReportPassRoll`
    #[serde(rename = "PASS_ROLL")]
    PassRoll {
        #[serde(rename = "playerId")]
        player_id: String,
        successful: bool,
        roll: i32,
        #[serde(rename = "minimumRoll")]
        minimum_roll: i32,
        #[serde(rename = "reRolled")]
        re_rolled: bool,
        #[serde(rename = "passingDistance")]
        passing_distance: Option<String>,
        #[serde(rename = "passResult")]
        pass_result: String,
        #[serde(rename = "hailMaryPass")]
        hail_mary_pass: bool,
        bomb: bool,
    },

    /// Java: `ReportInjury`
    #[serde(rename = "INJURY")]
    Injury {
        #[serde(rename = "attackerId")]
        attacker_id: Option<String>,
        #[serde(rename = "defenderId")]
        defender_id: String,
        #[serde(rename = "armorRoll")]
        armor_roll: Vec<i32>,
        #[serde(rename = "injuryRoll")]
        injury_roll: Vec<i32>,
        #[serde(rename = "armorBroken")]
        armor_broken: bool,
        injury: Option<u32>,
        #[serde(rename = "casualtyRoll")]
        casualty_roll: Vec<i32>,
        #[serde(rename = "seriousInjury")]
        serious_injury: Option<String>,
    },

    /// Java: `ReportReRoll`
    #[serde(rename = "RE_ROLL")]
    ReRoll {
        #[serde(rename = "teamId")]
        team_id: String,
        source: String,
        #[serde(rename = "reRolledAction")]
        re_rolled_action: Option<String>,
    },

    /// Java: `ReportSkillUse`
    #[serde(rename = "SKILL_USE")]
    SkillUse {
        #[serde(rename = "playerId")]
        player_id: Option<String>,
        skill: String,
        used: bool,
        #[serde(rename = "skillUse")]
        skill_use: String,
    },

    /// Java: `ReportPlayerAction` — player activates with an action type.
    #[serde(rename = "PLAYER_ACTION")]
    PlayerAction {
        #[serde(rename = "playerId")]
        player_id: String,
        #[serde(rename = "playerAction")]
        player_action: String,
    },

    /// Java: `ReportTurnEnd`
    #[serde(rename = "TURN_END")]
    TurnEnd {
        #[serde(rename = "teamId")]
        team_id: String,
        #[serde(rename = "turnNr")]
        turn_nr: i32,
    },

    /// Java: `ReportKickoffResult`
    #[serde(rename = "KICKOFF_RESULT")]
    KickoffResult { result: String },

    /// Java: `ReportCoinThrow`
    #[serde(rename = "COIN_THROW")]
    CoinThrow {
        #[serde(rename = "homeWon")]
        home_won: bool,
    },

    /// Java: `ReportReceiveChoice`
    #[serde(rename = "RECEIVE_CHOICE")]
    ReceiveChoice {
        #[serde(rename = "teamId")]
        team_id: String,
        receive: bool,
    },

    /// Java: `ReportTouchdown`
    #[serde(rename = "TOUCHDOWN")]
    Touchdown {
        #[serde(rename = "playerId")]
        player_id: String,
    },

    /// Java: `ReportFoul`
    #[serde(rename = "FOUL")]
    Foul {
        #[serde(rename = "defenderId")]
        defender_id: String,
    },

    /// Java: `ReportWeatherChange`
    #[serde(rename = "WEATHER_CHANGE")]
    WeatherChange { weather: String },

    // ── Phase ZV: extended coverage ─────────────────────────────────────────

    /// Java: `ReportAlwaysHungryRoll`
    #[serde(rename = "ALWAYS_HUNGRY_ROLL")]
    AlwaysHungryRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportTeamCaptainRoll`
    #[serde(rename = "TEAM_CAPTAIN_ROLL")]
    TeamCaptainRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, #[serde(rename = "rerollSaved")] reroll_saved: bool },

    /// Java: `ReportAnimosityRoll`
    #[serde(rename = "ANIMOSITY_ROLL")]
    AnimosityRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportArgueTheCallRoll`
    #[serde(rename = "ARGUE_THE_CALL")]
    ArgueTheCall { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportBloodLustRoll`
    #[serde(rename = "BLOOD_LUST_ROLL")]
    BloodLustRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportBribesRoll`
    #[serde(rename = "BRIBES_ROLL")]
    BribesRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportBalefulHexRoll`
    #[serde(rename = "BALEFUL_HEX")]
    BalefulHexRoll {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "targetId")] target_id: String,
        roll: i32, successful: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportChainsawRoll`
    #[serde(rename = "CHAINSAW_ROLL")]
    ChainsawRoll {
        #[serde(rename = "playerId")] player_id: String,
        roll: i32,
        #[serde(rename = "minimumRoll")] minimum_roll: i32,
        successful: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportLookIntoMyEyesRoll`
    #[serde(rename = "LOOK_INTO_MY_EYES_ROLL")]
    LookIntoMyEyesRoll {
        #[serde(rename = "playerId")] player_id: String,
        roll: i32, successful: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportPickMeUpRoll`
    #[serde(rename = "PICK_ME_UP")]
    PickMeUpRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportConfusionRoll`
    #[serde(rename = "CONFUSION_ROLL")]
    ConfusionRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, confused: bool },

    /// Java: `ReportDauntlessRoll`
    #[serde(rename = "DAUNTLESS_ROLL")]
    DauntlessRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportEscapeRoll`
    #[serde(rename = "ESCAPE_ROLL")]
    EscapeRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportFoulAppearanceRoll`
    #[serde(rename = "FOUL_APPEARANCE_ROLL")]
    FoulAppearanceRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, failed: bool },

    /// Java: `ReportHypnoticGazeRoll`
    #[serde(rename = "HYPNOTIC_GAZE_ROLL")]
    HypnoticGazeRoll {
        #[serde(rename = "playerId")] player_id: String,
        #[serde(rename = "targetId")] target_id: String,
        roll: i32, successful: bool,
    },

    /// Java: `ReportInterceptionRoll`
    #[serde(rename = "INTERCEPTION_ROLL")]
    InterceptionRoll { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "minimumRoll")] minimum_roll: i32, roll: i32, successful: bool },

    /// Java: `ReportJumpRoll`
    #[serde(rename = "JUMP_ROLL")]
    JumpRoll { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "minimumRoll")] minimum_roll: i32, roll: i32, successful: bool },

    /// Java: `ReportJumpUpRoll`
    #[serde(rename = "JUMP_UP_ROLL")]
    JumpUpRoll { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "minimumRoll")] minimum_roll: i32, roll: i32, successful: bool },

    /// Java: `ReportMasterChefRoll`
    #[serde(rename = "MASTER_CHEF_ROLL")]
    MasterChefRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, #[serde(rename = "rerollsStolen")] rerolls_stolen: i32 },

    /// Java: `ReportPilingOn`
    #[serde(rename = "PILING_ON")]
    PilingOn {
        #[serde(rename = "playerId")] player_id: String,
        #[serde(rename = "targetId")] target_id: String,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportPrayerRoll`
    #[serde(rename = "PRAYER_ROLL")]
    PrayerRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, #[serde(rename = "prayerId")] prayer_id: String },

    /// Java: `ReportRegenerationRoll`
    #[serde(rename = "REGENERATION_ROLL")]
    RegenerationRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportRightStuffRoll`
    #[serde(rename = "RIGHT_STUFF_ROLL")]
    RightStuffRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportSafeThrowRoll`
    #[serde(rename = "SAFE_THROW_ROLL")]
    SafeThrowRoll { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportStandUpRoll`
    #[serde(rename = "STAND_UP_ROLL")]
    StandUpRoll { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "minimumRoll")] minimum_roll: i32, roll: i32, successful: bool },

    /// Java: `ReportSwarmingPlayersRoll`
    #[serde(rename = "SWARMING_PLAYERS_ROLL")]
    SwarmingPlayersRoll { #[serde(rename = "teamId")] team_id: String, roll: i32 },

    /// Java: `ReportThrowTeamMateRoll`
    #[serde(rename = "THROW_TEAM_MATE_ROLL")]
    ThrowTeamMateRoll {
        #[serde(rename = "throwerId")] thrower_id: String,
        #[serde(rename = "thrownId")] thrown_id: String,
        roll: i32,
        #[serde(rename = "passResult")] pass_result: String,
    },

    /// Java: `ReportWeepingDaggerRoll`
    #[serde(rename = "WEEPING_DAGGER_ROLL")]
    WeepingDaggerRoll { #[serde(rename = "playerId")] player_id: String, roll: i32 },

    /// Java: `ReportSpellEffectRoll`
    #[serde(rename = "SPELL_EFFECT_ROLL")]
    SpellEffectRoll { roll: i32 },

    /// Java: `ReportBreatheFire`
    #[serde(rename = "BREATHE_FIRE")]
    BreatheFireRoll {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        roll: i32,
        #[serde(rename = "knockDown")] knock_down: bool,
        prone: bool,
        failure: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportProjectileVomit`
    #[serde(rename = "PROJECTILE_VOMIT")]
    ProjectileVomitRoll {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        roll: i32, successful: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportTrapDoor`
    #[serde(rename = "TRAP_DOOR")]
    TrapDoor { #[serde(rename = "playerId")] player_id: String, roll: i32, escaped: bool },

    /// Java: `ReportPushback`
    #[serde(rename = "PUSHBACK")]
    Pushback {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: String,
        squares: Vec<ffb_model::types::FieldCoordinate>,
    },

    /// Java: `ReportScatterBall`
    #[serde(rename = "SCATTER_BALL")]
    ScatterBall { from: ffb_model::types::FieldCoordinate, directions: Vec<i32> },

    /// Java: `ReportScatterPlayer`
    #[serde(rename = "SCATTER_PLAYER")]
    ScatterPlayer { #[serde(rename = "playerId")] player_id: String, coords: Vec<ffb_model::types::FieldCoordinate> },

    /// Java: `ReportSwoopPlayer`
    #[serde(rename = "SWOOP_PLAYER")]
    SwoopPlayer { #[serde(rename = "playerId")] player_id: String, coord: ffb_model::types::FieldCoordinate },

    /// Java: `ReportThrowIn`
    #[serde(rename = "THROW_IN")]
    ThrowIn { coord: ffb_model::types::FieldCoordinate, direction: i32, distance: i32 },

    /// Java: `ReportHandOver`
    #[serde(rename = "HAND_OVER")]
    HandOver { #[serde(rename = "fromId")] from_id: String, #[serde(rename = "toId")] to_id: String },

    /// Java: `ReportKickoffScatter`
    #[serde(rename = "KICKOFF_SCATTER")]
    KickoffScatter { start: ffb_model::types::FieldCoordinate, direction: i32, distance: i32 },

    /// Java: `ReportPassDeviate`
    #[serde(rename = "PASS_DEVIATE")]
    PassDeviate { from: ffb_model::types::FieldCoordinate, #[serde(rename = "scatterDirections")] scatter_directions: Vec<i32> },

    /// Java: `ReportKickoffPitchInvasion` (BB2020/BB2025 simple variant)
    #[serde(rename = "KICKOFF_PITCH_INVASION")]
    KickoffPitchInvasion { #[serde(rename = "homeRoll")] home_roll: i32, #[serde(rename = "awayRoll")] away_roll: i32 },

    /// Java: `ReportKickoffRiot`
    #[serde(rename = "KICKOFF_RIOT")]
    KickoffRiot { #[serde(rename = "turnModifier")] turn_modifier: i32, roll: i32 },

    /// Java: `ReportKickoffThrowARock`
    #[serde(rename = "KICKOFF_THROW_A_ROCK")]
    KickoffThrowARock { #[serde(rename = "playerId")] player_id: Option<String> },

    /// Java: `ReportRiotousRookies`
    #[serde(rename = "RIOTOUS_ROOKIES")]
    RiotousRookies { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "playerCount")] player_count: i32 },

    /// Java: `ReportHitAndRun`
    #[serde(rename = "HIT_AND_RUN")]
    HitAndRun { #[serde(rename = "playerId")] player_id: String, direction: String },

    /// Java: `ReportKickTeamMateFumble`
    #[serde(rename = "KICK_TEAM_MATE_FUMBLE")]
    KickTeamMateFumble,

    /// Java: `ReportPrayerAmount`
    #[serde(rename = "PRAYER_AMOUNT")]
    PrayerAmount {
        #[serde(rename = "tvHome")] tv_home: i32,
        #[serde(rename = "tvAway")] tv_away: i32,
        #[serde(rename = "prayerAmount")] prayer_amount: i32,
        #[serde(rename = "homeTeamReceivesPrayers")] home_team_receives_prayers: bool,
    },

    /// Java: `ReportBiteSpectator`
    #[serde(rename = "BITE_SPECTATOR")]
    BiteSpectator { #[serde(rename = "playerId")] player_id: String },

    /// Java: `ReportCardsAndInducementsBought`
    #[serde(rename = "CARDS_AND_INDUCEMENTS_BOUGHT")]
    CardsAndInducementsBought {
        #[serde(rename = "teamId")] team_id: String,
        cards: i32,
        inducements: i32,
        stars: i32,
        mercenaries: i32,
        gold: i32,
        #[serde(rename = "newTv")] new_tv: i32,
    },

    /// Java: `ReportKickoffSequenceActivationsExhausted`
    #[serde(rename = "KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED")]
    KickoffSequenceActivationsExhausted { #[serde(rename = "limitReached")] limit_reached: bool },

    /// Java: `ReportSolidDefenceRoll`
    #[serde(rename = "SOLID_DEFENCE_ROLL")]
    SolidDefenceRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, amount: i32 },

    /// Java: `ReportCheeringFans` (Kickoff Cheering Fans)
    #[serde(rename = "KICKOFF_CHEERING_FANS")]
    CheeringFans { #[serde(rename = "homeRoll")] home_roll: i32, #[serde(rename = "awayRoll")] away_roll: i32 },

    /// Java: `ReportKickoffExtraReRoll`
    #[serde(rename = "KICKOFF_EXTRA_RE_ROLL")]
    KickoffExtraReRoll { #[serde(rename = "teamId")] team_id: String },

    /// Java: `ReportQuickSnapRoll`
    #[serde(rename = "QUICK_SNAP_ROLL")]
    QuickSnapRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, amount: i32 },

    /// Java: `ReportKickoffTimeout`
    #[serde(rename = "KICKOFF_TIMEOUT")]
    KickoffTimeout { #[serde(rename = "turnNumber")] turn_number: i32, #[serde(rename = "turnModifier")] turn_modifier: i32 },

    /// Java: `ReportKickoffOfficiousRef`
    #[serde(rename = "KICKOFF_OFFICIOUS_REF")]
    KickoffOfficiousRef {
        #[serde(rename = "rollHome")] roll_home: i32,
        #[serde(rename = "rollAway")] roll_away: i32,
        #[serde(rename = "playerIds")] player_ids: Vec<String>,
    },

    /// Java: `ReportKickoffDodgySnack`
    #[serde(rename = "KICKOFF_DODGY_SNACK")]
    KickoffDodgySnack {
        #[serde(rename = "rollHome")] roll_home: i32,
        #[serde(rename = "rollAway")] roll_away: i32,
        #[serde(rename = "playerIds")] player_ids: Vec<String>,
    },

    /// Java: `ReportDodgySnackRoll`
    #[serde(rename = "DODGY_SNACK_ROLL")]
    DodgySnackRoll { #[serde(rename = "playerId")] player_id: String, roll: i32 },

    /// Java: `ReportThrowAtPlayer`
    #[serde(rename = "THROW_AT_PLAYER")]
    ThrowAtPlayer { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportFumblerooskie`
    #[serde(rename = "FUMBLEROOSKIE")]
    Fumblerooskie { #[serde(rename = "playerId")] player_id: String, used: bool },

    /// Java: `ReportAllYouCanEat`
    #[serde(rename = "ALL_YOU_CAN_EAT")]
    AllYouCanEatRoll {
        #[serde(rename = "playerId")] player_id: String,
        roll: i32,
        #[serde(rename = "minimumRoll")] minimum_roll: i32,
        successful: bool,
        #[serde(rename = "reRolled")] re_rolled: bool,
    },

    /// Java: `ReportKickoffExtraReRoll` (BB2016 combined Cheering Fans / Brilliant Coaching variant)
    #[serde(rename = "KICKOFF_EXTRA_RE_ROLL")]
    KickoffExtraReRollBb2016 {
        #[serde(rename = "kickoffResult")] kickoff_result: String,
        #[serde(rename = "rollHome")] roll_home: i32,
        #[serde(rename = "homeGainsReroll")] home_gains_reroll: bool,
        #[serde(rename = "rollAway")] roll_away: i32,
        #[serde(rename = "awayGainsReroll")] away_gains_reroll: bool,
    },

    /// Java: `ReportKickoffThrowARock` (BB2016 multi-player variant)
    #[serde(rename = "KICKOFF_THROW_A_ROCK")]
    KickoffThrowARockBb2016 {
        #[serde(rename = "rollHome")] roll_home: i32,
        #[serde(rename = "rollAway")] roll_away: i32,
        #[serde(rename = "playerIds")] player_ids: Vec<String>,
    },

    /// Java: `ReportKickoffPitchInvasion` (BB2016 per-player roll variant)
    #[serde(rename = "KICKOFF_PITCH_INVASION")]
    KickoffPitchInvasionBb2016 {
        #[serde(rename = "rollsHome")] rolls_home: Vec<i32>,
        #[serde(rename = "affectedHome")] affected_home: Vec<bool>,
        #[serde(rename = "rollsAway")] rolls_away: Vec<i32>,
        #[serde(rename = "affectedAway")] affected_away: Vec<bool>,
    },

    /// Java: `ReportAnimalSavagery`
    #[serde(rename = "ANIMAL_SAVAGERY")]
    AnimalSavagery { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportLeader`
    #[serde(rename = "LEADER")]
    Leader { #[serde(rename = "playerId")] player_id: String, #[serde(rename = "rerollAvailable")] reroll_available: bool },

    /// Java: `ReportThenIStartedBlastin`
    #[serde(rename = "THEN_I_STARTED_BLASTIN")]
    ThenIStartedBlastin {
        #[serde(rename = "attackerId")] attacker_id: String,
        #[serde(rename = "defenderId")] defender_id: Option<String>,
        roll: i32, successful: bool, fumble: bool,
    },

    /// Java: `ReportApothecaryRoll`
    #[serde(rename = "APOTHECARY_ROLL")]
    ApothecaryRoll {
        #[serde(rename = "playerId")] player_id: String,
        roll: Option<i32>,
        #[serde(rename = "newState")] new_state: Option<u16>,
        #[serde(rename = "newSeriousInjury")] new_serious_injury: Option<String>,
    },

    /// Java: `ReportApothecaryChoice`
    #[serde(rename = "APOTHECARY_CHOICE")]
    ApothecaryChoice { #[serde(rename = "playerId")] player_id: String, healed: bool },

    /// Java: `ReportSelectBlitzTarget`
    #[serde(rename = "SELECT_BLITZ_TARGET")]
    SelectBlitzTarget { #[serde(rename = "attackerId")] attacker_id: String, #[serde(rename = "defenderId")] defender_id: String },

    /// Java: `ReportSelectGazeTarget`
    #[serde(rename = "SELECT_GAZE_TARGET")]
    SelectGazeTarget { #[serde(rename = "attackerId")] attacker_id: String, #[serde(rename = "defenderId")] defender_id: String },

    /// Java: `ReportCardDeactivated`
    #[serde(rename = "CARD_DEACTIVATED")]
    CardDeactivated { #[serde(rename = "cardId")] card_id: String },

    /// Java: `ReportCardEffectRoll`
    #[serde(rename = "CARD_EFFECT_ROLL")]
    CardEffectRoll { #[serde(rename = "cardId")] card_id: String, roll: i32, effect: String },

    /// Java: `ReportDefectingPlayers`
    #[serde(rename = "DEFECTING_PLAYERS")]
    DefectingPlayers { #[serde(rename = "playerIds")] player_ids: Vec<String> },

    /// Java: `ReportPassBlock`
    #[serde(rename = "PASS_BLOCK")]
    PassBlock { #[serde(rename = "playerId")] player_id: Option<String> },

    /// Java: `ReportPettyCash`
    #[serde(rename = "PETTY_CASH")]
    PettyCash { #[serde(rename = "teamId")] team_id: String, amount: i32 },

    /// Java: `ReportPlayCard`
    #[serde(rename = "PLAY_CARD")]
    PlayCard { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "cardId")] card_id: String },

    /// Java: `ReportSecretWeaponBan`
    #[serde(rename = "SECRET_WEAPON_BAN")]
    SecretWeaponBan { #[serde(rename = "playerId")] player_id: String },

    /// Java: `ReportWizardUse`
    #[serde(rename = "WIZARD_USE")]
    WizardUse {
        #[serde(rename = "teamId")] team_id: String,
        spell: String,
        coord: Option<ffb_model::types::FieldCoordinate>,
    },

    /// Java: `ReportBombExplodesAfterCatch`
    #[serde(rename = "BOMB_EXPLODES_AFTER_CATCH")]
    BombExplodesAfterCatch { #[serde(rename = "playerId")] player_id: String, coord: ffb_model::types::FieldCoordinate },

    /// Java: `ReportBombOutOfBounds`
    #[serde(rename = "BOMB_OUT_OF_BOUNDS")]
    BombOutOfBounds { coord: ffb_model::types::FieldCoordinate },

    /// Java: `ReportThrownKeg`
    #[serde(rename = "THROWN_KEG")]
    KegThrow {
        #[serde(rename = "throwerId")] thrower_id: String,
        #[serde(rename = "targetId")] target_id: Option<String>,
        roll: i32, successful: bool, fumble: bool,
    },

    /// Java: `ReportInducement`
    #[serde(rename = "INDUCEMENT")]
    Inducement { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "inducementType")] inducement_type: String, value: i32 },

    /// Java: `ReportPumpUpTheCrowdReRoll`
    #[serde(rename = "PUMP_UP_THE_CROWD_RE_ROLL")]
    PumpUpTheCrowdReRoll { #[serde(rename = "playerId")] player_id: String },

    /// Java: `ReportReferee`
    #[serde(rename = "REFEREE")]
    RefereeSpotsFoul {
        #[serde(rename = "refereeSpotsFoul")] referee_spots_foul: bool,
        #[serde(rename = "underScrutiny")] under_scrutiny: bool,
    },

    /// Java: `ReportBiasedRef`
    #[serde(rename = "BIASED_REF")]
    BiasedRefRoll { roll: i32, #[serde(rename = "refereeSpotsFoul")] referee_spots_foul: bool },

    /// Java: `ReportDoubleHiredStarPlayer`
    #[serde(rename = "DOUBLE_HIRED_STAR_PLAYER")]
    DoubleHiredStarPlayer,

    /// Java: `ReportGameOptions`
    #[serde(rename = "GAME_OPTIONS")]
    GameOptions { #[serde(rename = "optionsSnapshot")] options_snapshot: std::collections::HashMap<String, String> },

    /// Java: `ReportStartHalf`
    #[serde(rename = "START_HALF")]
    StartHalf { half: i32 },

    /// Java: `ReportTimeoutEnforced`
    #[serde(rename = "TIMEOUT_ENFORCED")]
    TimeoutEnforced { #[serde(rename = "teamId")] team_id: String },

    /// Java: `ReportWinningsRoll`
    #[serde(rename = "WINNINGS_ROLL")]
    WinningsRoll { #[serde(rename = "teamId")] team_id: String, base: i32, roll: i32, total: i32 },

    /// Java: `ReportMostValuablePlayers` — single-player MVP roll surfaced from the engine.
    #[serde(rename = "MOST_VALUABLE_PLAYERS")]
    MvpRoll { #[serde(rename = "teamId")] team_id: String, #[serde(rename = "playerId")] player_id: String, spp: i32 },

    /// Java: `ReportBlitzRoll`
    #[serde(rename = "BLITZ_ROLL")]
    BlitzRoll { #[serde(rename = "teamId")] team_id: String, roll: i32, limit: i32 },

    /// Java: `ReportThrowAtStallingPlayer`
    #[serde(rename = "THROW_AT_STALLING_PLAYER")]
    ThrowAtStallingPlayer { #[serde(rename = "playerId")] player_id: String, roll: i32, successful: bool },

    /// Java: `ReportNoPlayersToField`
    #[serde(rename = "NO_PLAYERS_TO_FIELD")]
    NoPlayersToField { #[serde(rename = "teamId")] team_id: Option<String> },

    /// Java: `ReportPlayerEvent`
    #[serde(rename = "PLAYER_EVENT")]
    PlayerNote { #[serde(rename = "playerId")] player_id: String, note: String },
}

// ── ModelChangeList ────────────────────────────────────────────────────────

/// Java: `ModelChangeList` → JSON `{ "modelChangeArray": [...] }`
#[derive(Serialize)]
pub struct WireModelChangeList {
    #[serde(rename = "modelChangeArray")]
    pub model_change_array: Vec<WireModelChange>,
}

/// Java: `ModelChange` → JSON `{ "modelChangeId": "...", "modelChangeKey": "...", "modelChangeValue": ... }`
#[derive(Serialize)]
pub struct WireModelChange {
    #[serde(rename = "modelChangeId")]
    pub model_change_id: String,
    #[serde(rename = "modelChangeKey")]
    pub model_change_key: String,
    #[serde(rename = "modelChangeValue")]
    pub model_change_value: serde_json::Value,
}

// ── ServerCommandModelSync outgoing ────────────────────────────────────────

/// Outgoing `serverModelSync` command matching Java's wire format exactly.
///
/// Java: `ServerCommandModelSync`
#[derive(Serialize)]
pub struct OutgoingModelSync {
    #[serde(rename = "netCommandId")]
    pub net_command_id: &'static str,
    #[serde(rename = "commandNr")]
    pub command_nr: i64,
    #[serde(rename = "modelChangeList")]
    pub model_change_list: WireModelChangeList,
    #[serde(rename = "reportList")]
    pub report_list: WireReportList,
    pub animation: serde_json::Value,
    pub sound: Option<String>,
    #[serde(rename = "gameTime")]
    pub game_time: i64,
    #[serde(rename = "turnTime")]
    pub turn_time: i64,
}

impl OutgoingModelSync {
    pub fn new(command_nr: i64, reports: Vec<WireReport>) -> Self {
        Self {
            net_command_id: "serverModelSync",
            command_nr,
            model_change_list: WireModelChangeList { model_change_array: vec![] },
            report_list: WireReportList { reports },
            animation: serde_json::Value::Null,
            sound: None,
            game_time: 0,
            turn_time: 0,
        }
    }
}

// ── GameEvent → WireReport conversion ─────────────────────────────────────

/// Convert a single `GameEvent` to a `WireReport` if there is a matching
/// report type.  Returns `None` for events that have no report equivalent
/// (e.g. internal state changes that the client doesn't display).
pub fn event_to_report(event: &GameEvent) -> Option<WireReport> {
    match event {
        GameEvent::BlockRoll { attacker_id, defender_id, nr_of_dice, dice, selected_index, own_choice, rerolled, .. } =>
            Some(WireReport::BlockRoll {
                attacker_id: attacker_id.clone(),
                defender_id: defender_id.clone(),
                nr_of_dice: *nr_of_dice,
                dice: dice.clone(),
                selected_die: *selected_index,
                own_choice: *own_choice,
                re_rolled: *rerolled,
            }),
        GameEvent::Block { defender_id } =>
            Some(WireReport::Block { defender_id: defender_id.clone() }),
        GameEvent::DodgeRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::DodgeRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::GoForItRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::GoForItRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::PickupRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::PickupRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::CatchRoll { player_id, target, roll, success, rerolled } =>
            Some(WireReport::CatchRoll {
                player_id: player_id.clone(),
                successful: *success,
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
            }),
        GameEvent::PassRoll { player_id, target, distance, roll, result, rerolled } =>
            Some(WireReport::PassRoll {
                player_id: player_id.clone(),
                successful: !matches!(result, ffb_model::enums::PassResult::Fumble),
                roll: *roll,
                minimum_roll: *target,
                re_rolled: *rerolled,
                passing_distance: Some(format!("{:?}", distance)),
                pass_result: format!("{:?}", result),
                hail_mary_pass: false,
                bomb: false,
            }),
        GameEvent::Injury { player_id, armor_roll, injury_roll, serious_injury, was_ko, was_cas } =>
            Some(WireReport::Injury {
                attacker_id: None,
                defender_id: player_id.clone(),
                armor_roll: armor_roll.map(|r| r.to_vec()).unwrap_or_default(),
                injury_roll: injury_roll.map(|r| r.to_vec()).unwrap_or_default(),
                armor_broken: *was_ko || *was_cas,
                injury: None,
                casualty_roll: vec![],
                serious_injury: serious_injury.as_ref().map(|s| format!("{:?}", s)),
            }),
        GameEvent::ReRoll { team_id, source, rerolled_action } =>
            Some(WireReport::ReRoll {
                team_id: team_id.clone(),
                source: source.name.clone(),
                re_rolled_action: Some(rerolled_action.clone()),
            }),
        GameEvent::SkillUse { player_id, skill_id, used } =>
            Some(WireReport::SkillUse {
                player_id: Some(player_id.clone()),
                skill: skill_id.to_string(),
                used: *used,
                skill_use: if *used { "USED".to_string() } else { "DECLINED".to_string() },
            }),
        GameEvent::PlayerAction { player_id, action } =>
            Some(WireReport::PlayerAction {
                player_id: player_id.clone(),
                player_action: format!("{:?}", action),
            }),
        GameEvent::TurnEnd { team_id, turn_nr } =>
            Some(WireReport::TurnEnd { team_id: team_id.clone(), turn_nr: *turn_nr }),
        GameEvent::KickoffResultEvent { result } =>
            Some(WireReport::KickoffResult { result: format!("{:?}", result) }),
        GameEvent::CoinThrow { home_won } =>
            Some(WireReport::CoinThrow { home_won: *home_won }),
        GameEvent::ReceiveChoice { team_id, receive } =>
            Some(WireReport::ReceiveChoice { team_id: team_id.clone(), receive: *receive }),
        GameEvent::Touchdown { player_id, .. } =>
            Some(WireReport::Touchdown { player_id: player_id.clone() }),
        GameEvent::Foul { defender_id, .. } =>
            Some(WireReport::Foul { defender_id: defender_id.clone() }),
        GameEvent::WeatherChange { weather } =>
            Some(WireReport::WeatherChange { weather: format!("{:?}", weather) }),

        // ── Phase ZV: extended coverage ──────────────────────────────────────
        GameEvent::AlwaysHungry { player_id, roll, success } =>
            Some(WireReport::AlwaysHungryRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::TeamCaptainRoll { team_id, roll, reroll_saved } =>
            Some(WireReport::TeamCaptainRoll { team_id: team_id.clone(), roll: *roll, reroll_saved: *reroll_saved }),
        GameEvent::AnimosityRoll { player_id, roll, success } =>
            Some(WireReport::AnimosityRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::ArgueTheCall { player_id, roll, success } =>
            Some(WireReport::ArgueTheCall { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::BloodLustRoll { player_id, roll, success } =>
            Some(WireReport::BloodLustRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::BribesRoll { player_id, roll, success } =>
            Some(WireReport::BribesRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::BalefulHexRoll { attacker_id, target_id, roll, success, rerolled } =>
            Some(WireReport::BalefulHexRoll { attacker_id: attacker_id.clone(), target_id: target_id.clone(), roll: *roll, successful: *success, re_rolled: *rerolled }),
        GameEvent::ChainsawRoll { player_id, roll, minimum_roll, success, rerolled } =>
            Some(WireReport::ChainsawRoll { player_id: player_id.clone(), roll: *roll, minimum_roll: *minimum_roll, successful: *success, re_rolled: *rerolled }),
        GameEvent::LookIntoMyEyesRoll { player_id, roll, success, rerolled } =>
            Some(WireReport::LookIntoMyEyesRoll { player_id: player_id.clone(), roll: *roll, successful: *success, re_rolled: *rerolled }),
        GameEvent::PickMeUpRoll { player_id, roll, success } =>
            Some(WireReport::PickMeUpRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::ConfusionRoll { player_id, roll, confused } =>
            Some(WireReport::ConfusionRoll { player_id: player_id.clone(), roll: *roll, confused: *confused }),
        GameEvent::DauntlessRoll { player_id, roll, success } =>
            Some(WireReport::DauntlessRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::EscapeRoll { player_id, roll, success } =>
            Some(WireReport::EscapeRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::FoulAppearanceRoll { player_id, roll, failed } =>
            Some(WireReport::FoulAppearanceRoll { player_id: player_id.clone(), roll: *roll, failed: *failed }),
        GameEvent::HypnoticGazeRoll { player_id, target_id, roll, success } =>
            Some(WireReport::HypnoticGazeRoll { player_id: player_id.clone(), target_id: target_id.clone(), roll: *roll, successful: *success }),
        GameEvent::InterceptionRoll { player_id, target, roll, success } =>
            Some(WireReport::InterceptionRoll { player_id: player_id.clone(), minimum_roll: *target, roll: *roll, successful: *success }),
        GameEvent::JumpRoll { player_id, target, roll, success } =>
            Some(WireReport::JumpRoll { player_id: player_id.clone(), minimum_roll: *target, roll: *roll, successful: *success }),
        GameEvent::JumpUpRoll { player_id, target, roll, success } =>
            Some(WireReport::JumpUpRoll { player_id: player_id.clone(), minimum_roll: *target, roll: *roll, successful: *success }),
        GameEvent::MasterChefRoll { team_id, roll, rerolls_stolen } =>
            Some(WireReport::MasterChefRoll { team_id: team_id.clone(), roll: *roll, rerolls_stolen: *rerolls_stolen }),
        GameEvent::PilingOn { player_id, target_id, rerolled } =>
            Some(WireReport::PilingOn { player_id: player_id.clone(), target_id: target_id.clone(), re_rolled: *rerolled }),
        GameEvent::PrayerRoll { team_id, roll, prayer_id } =>
            Some(WireReport::PrayerRoll { team_id: team_id.clone(), roll: *roll, prayer_id: prayer_id.clone() }),
        GameEvent::RegenerationRoll { player_id, roll, success } =>
            Some(WireReport::RegenerationRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::RightStuffRoll { player_id, roll, success } =>
            Some(WireReport::RightStuffRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::SafeThrowRoll { player_id, roll, success } =>
            Some(WireReport::SafeThrowRoll { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::StandUpRoll { player_id, target, roll, success } =>
            Some(WireReport::StandUpRoll { player_id: player_id.clone(), minimum_roll: *target, roll: *roll, successful: *success }),
        GameEvent::SwarmingPlayersRoll { team_id, roll } =>
            Some(WireReport::SwarmingPlayersRoll { team_id: team_id.clone(), roll: *roll }),
        GameEvent::ThrowTeamMateRoll { thrower_id, thrown_id, roll, result } =>
            Some(WireReport::ThrowTeamMateRoll { thrower_id: thrower_id.clone(), thrown_id: thrown_id.clone(), roll: *roll, pass_result: format!("{:?}", result) }),
        GameEvent::WeepingDaggerRoll { player_id, roll } =>
            Some(WireReport::WeepingDaggerRoll { player_id: player_id.clone(), roll: *roll }),
        GameEvent::SpellEffectRoll { roll } =>
            Some(WireReport::SpellEffectRoll { roll: *roll }),
        GameEvent::BreatheFireRoll { attacker_id, defender_id, roll, knock_down, prone, failure, rerolled } =>
            Some(WireReport::BreatheFireRoll { attacker_id: attacker_id.clone(), defender_id: defender_id.clone(), roll: *roll, knock_down: *knock_down, prone: *prone, failure: *failure, re_rolled: *rerolled }),
        GameEvent::ProjectileVomitRoll { attacker_id, defender_id, roll, success, rerolled } =>
            Some(WireReport::ProjectileVomitRoll { attacker_id: attacker_id.clone(), defender_id: defender_id.clone(), roll: *roll, successful: *success, re_rolled: *rerolled }),
        GameEvent::TrapDoor { player_id, roll, escaped } =>
            Some(WireReport::TrapDoor { player_id: player_id.clone(), roll: *roll, escaped: *escaped }),
        GameEvent::Pushback { attacker_id, defender_id, squares } =>
            Some(WireReport::Pushback { attacker_id: attacker_id.clone(), defender_id: defender_id.clone(), squares: squares.clone() }),
        GameEvent::ScatterBall { from, directions } =>
            Some(WireReport::ScatterBall { from: *from, directions: directions.clone() }),
        GameEvent::ScatterPlayer { player_id, coords } =>
            Some(WireReport::ScatterPlayer { player_id: player_id.clone(), coords: coords.clone() }),
        GameEvent::SwoopPlayer { player_id, coord } =>
            Some(WireReport::SwoopPlayer { player_id: player_id.clone(), coord: *coord }),
        GameEvent::ThrowIn { coord, direction, distance } =>
            Some(WireReport::ThrowIn { coord: *coord, direction: *direction, distance: *distance }),
        GameEvent::HandOver { from_id, to_id } =>
            Some(WireReport::HandOver { from_id: from_id.clone(), to_id: to_id.clone() }),
        GameEvent::KickoffScatter { start, direction, distance } =>
            Some(WireReport::KickoffScatter { start: *start, direction: *direction, distance: *distance }),
        GameEvent::PassDeviate { from, scatter_directions } =>
            Some(WireReport::PassDeviate { from: *from, scatter_directions: scatter_directions.clone() }),
        GameEvent::KickoffPitchInvasion { home_roll, away_roll } =>
            Some(WireReport::KickoffPitchInvasion { home_roll: *home_roll, away_roll: *away_roll }),
        GameEvent::KickoffRiot { turn_modifier, roll } =>
            Some(WireReport::KickoffRiot { turn_modifier: *turn_modifier, roll: *roll }),
        GameEvent::KickoffThrowARock { player_id } =>
            Some(WireReport::KickoffThrowARock { player_id: player_id.clone() }),
        GameEvent::RiotousRookies { team_id, player_count } =>
            Some(WireReport::RiotousRookies { team_id: team_id.clone(), player_count: *player_count }),
        GameEvent::HitAndRun { player_id, direction } =>
            Some(WireReport::HitAndRun { player_id: player_id.clone(), direction: format!("{:?}", direction) }),
        GameEvent::KickTeamMateFumble =>
            Some(WireReport::KickTeamMateFumble),
        GameEvent::PrayerAmount { tv_home, tv_away, prayer_amount, home_team_receives_prayers } =>
            Some(WireReport::PrayerAmount { tv_home: *tv_home, tv_away: *tv_away, prayer_amount: *prayer_amount, home_team_receives_prayers: *home_team_receives_prayers }),
        GameEvent::BiteSpectator { player_id } =>
            Some(WireReport::BiteSpectator { player_id: player_id.clone() }),
        GameEvent::CardsAndInducementsBought { team_id, cards, inducements, stars, mercenaries, gold, new_tv } =>
            Some(WireReport::CardsAndInducementsBought { team_id: team_id.clone(), cards: *cards, inducements: *inducements, stars: *stars, mercenaries: *mercenaries, gold: *gold, new_tv: *new_tv }),
        GameEvent::KickoffSequenceActivationsExhausted { limit_reached } =>
            Some(WireReport::KickoffSequenceActivationsExhausted { limit_reached: *limit_reached }),
        GameEvent::SolidDefenceRoll { team_id, roll, amount } =>
            Some(WireReport::SolidDefenceRoll { team_id: team_id.clone(), roll: *roll, amount: *amount }),
        GameEvent::CheeringFans { home_roll, away_roll } =>
            Some(WireReport::CheeringFans { home_roll: *home_roll, away_roll: *away_roll }),
        GameEvent::KickoffExtraReRoll { team_id } =>
            Some(WireReport::KickoffExtraReRoll { team_id: team_id.clone() }),
        GameEvent::QuickSnapRoll { team_id, roll, amount } =>
            Some(WireReport::QuickSnapRoll { team_id: team_id.clone(), roll: *roll, amount: *amount }),
        GameEvent::KickoffTimeout { turn_number, turn_modifier } =>
            Some(WireReport::KickoffTimeout { turn_number: *turn_number, turn_modifier: *turn_modifier }),
        GameEvent::KickoffOfficiousRef { roll_home, roll_away, player_ids } =>
            Some(WireReport::KickoffOfficiousRef { roll_home: *roll_home, roll_away: *roll_away, player_ids: player_ids.clone() }),
        GameEvent::KickoffDodgySnack { roll_home, roll_away, player_ids } =>
            Some(WireReport::KickoffDodgySnack { roll_home: *roll_home, roll_away: *roll_away, player_ids: player_ids.clone() }),
        GameEvent::DodgySnackRoll { player_id, roll } =>
            Some(WireReport::DodgySnackRoll { player_id: player_id.clone(), roll: *roll }),
        GameEvent::ThrowAtPlayer { player_id, roll, successful } =>
            Some(WireReport::ThrowAtPlayer { player_id: player_id.clone(), roll: *roll, successful: *successful }),
        GameEvent::Fumblerooskie { player_id, used } =>
            Some(WireReport::Fumblerooskie { player_id: player_id.clone(), used: *used }),
        GameEvent::AllYouCanEatRoll { player_id, roll, minimum_roll, success, rerolled } =>
            Some(WireReport::AllYouCanEatRoll { player_id: player_id.clone(), roll: *roll, minimum_roll: *minimum_roll, successful: *success, re_rolled: *rerolled }),
        GameEvent::KickoffExtraReRollBb2016 { kickoff_result, roll_home, home_gains_reroll, roll_away, away_gains_reroll } =>
            Some(WireReport::KickoffExtraReRollBb2016 { kickoff_result: format!("{:?}", kickoff_result), roll_home: *roll_home, home_gains_reroll: *home_gains_reroll, roll_away: *roll_away, away_gains_reroll: *away_gains_reroll }),
        GameEvent::KickoffThrowARockBb2016 { roll_home, roll_away, player_ids } =>
            Some(WireReport::KickoffThrowARockBb2016 { roll_home: *roll_home, roll_away: *roll_away, player_ids: player_ids.clone() }),
        GameEvent::KickoffPitchInvasionBb2016 { rolls_home, affected_home, rolls_away, affected_away } =>
            Some(WireReport::KickoffPitchInvasionBb2016 { rolls_home: rolls_home.clone(), affected_home: affected_home.clone(), rolls_away: rolls_away.clone(), affected_away: affected_away.clone() }),
        GameEvent::AnimalSavagery { player_id, roll, success } =>
            Some(WireReport::AnimalSavagery { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::Leader { player_id, reroll_available } =>
            Some(WireReport::Leader { player_id: player_id.clone(), reroll_available: *reroll_available }),
        GameEvent::ThenIStartedBlastin { attacker_id, defender_id, roll, success, fumble } =>
            Some(WireReport::ThenIStartedBlastin { attacker_id: attacker_id.clone(), defender_id: defender_id.clone(), roll: *roll, successful: *success, fumble: *fumble }),
        GameEvent::ApothecaryRoll { player_id, roll, new_state, new_serious_injury } =>
            Some(WireReport::ApothecaryRoll { player_id: player_id.clone(), roll: *roll, new_state: *new_state, new_serious_injury: new_serious_injury.as_ref().map(|s| format!("{:?}", s)) }),
        GameEvent::ApothecaryChoice { player_id, healed } =>
            Some(WireReport::ApothecaryChoice { player_id: player_id.clone(), healed: *healed }),
        GameEvent::SelectBlitzTarget { attacker_id, defender_id } =>
            Some(WireReport::SelectBlitzTarget { attacker_id: attacker_id.clone(), defender_id: defender_id.clone() }),
        GameEvent::SelectGazeTarget { attacker_id, defender_id } =>
            Some(WireReport::SelectGazeTarget { attacker_id: attacker_id.clone(), defender_id: defender_id.clone() }),
        GameEvent::CardDeactivated { card_id } =>
            Some(WireReport::CardDeactivated { card_id: card_id.clone() }),
        GameEvent::CardEffectRoll { card_id, roll, effect } =>
            Some(WireReport::CardEffectRoll { card_id: card_id.clone(), roll: *roll, effect: effect.clone() }),
        GameEvent::DefectingPlayers { player_ids } =>
            Some(WireReport::DefectingPlayers { player_ids: player_ids.clone() }),
        GameEvent::PassBlock { player_id } =>
            Some(WireReport::PassBlock { player_id: player_id.clone() }),
        GameEvent::PettyCash { team_id, amount } =>
            Some(WireReport::PettyCash { team_id: team_id.clone(), amount: *amount }),
        GameEvent::PlayCard { team_id, card_id } =>
            Some(WireReport::PlayCard { team_id: team_id.clone(), card_id: card_id.clone() }),
        GameEvent::SecretWeaponBan { player_id } =>
            Some(WireReport::SecretWeaponBan { player_id: player_id.clone() }),
        GameEvent::WizardUse { team_id, spell, coord } =>
            Some(WireReport::WizardUse { team_id: team_id.clone(), spell: spell.clone(), coord: *coord }),
        GameEvent::BombExplodesAfterCatch { player_id, coord } =>
            Some(WireReport::BombExplodesAfterCatch { player_id: player_id.clone(), coord: *coord }),
        GameEvent::BombOutOfBounds { coord } =>
            Some(WireReport::BombOutOfBounds { coord: *coord }),
        GameEvent::KegThrow { thrower_id, target_id, roll, success, fumble } =>
            Some(WireReport::KegThrow { thrower_id: thrower_id.clone(), target_id: target_id.clone(), roll: *roll, successful: *success, fumble: *fumble }),
        GameEvent::Inducement { team_id, inducement_type, value } =>
            Some(WireReport::Inducement { team_id: team_id.clone(), inducement_type: inducement_type.clone(), value: *value }),
        GameEvent::PumpUpTheCrowdReRoll { player_id } =>
            Some(WireReport::PumpUpTheCrowdReRoll { player_id: player_id.clone() }),
        GameEvent::RefereeSpotsFoul { referee_spots_foul, under_scrutiny } =>
            Some(WireReport::RefereeSpotsFoul { referee_spots_foul: *referee_spots_foul, under_scrutiny: *under_scrutiny }),
        GameEvent::BiasedRefRoll { roll, referee_spots_foul } =>
            Some(WireReport::BiasedRefRoll { roll: *roll, referee_spots_foul: *referee_spots_foul }),
        GameEvent::DoubleHiredStarPlayer =>
            Some(WireReport::DoubleHiredStarPlayer),
        GameEvent::GameOptions { options_snapshot } =>
            Some(WireReport::GameOptions { options_snapshot: options_snapshot.clone() }),
        GameEvent::StartHalf { half } =>
            Some(WireReport::StartHalf { half: *half }),
        GameEvent::TimeoutEnforced { team_id } =>
            Some(WireReport::TimeoutEnforced { team_id: team_id.clone() }),
        GameEvent::WinningsRoll { team_id, base, roll, total } =>
            Some(WireReport::WinningsRoll { team_id: team_id.clone(), base: *base, roll: *roll, total: *total }),
        GameEvent::MvpRoll { team_id, player_id, spp } =>
            Some(WireReport::MvpRoll { team_id: team_id.clone(), player_id: player_id.clone(), spp: *spp }),
        GameEvent::BlitzRoll { team_id, roll, limit } =>
            Some(WireReport::BlitzRoll { team_id: team_id.clone(), roll: *roll, limit: *limit }),
        GameEvent::ThrowAtStallingPlayer { player_id, roll, success } =>
            Some(WireReport::ThrowAtStallingPlayer { player_id: player_id.clone(), roll: *roll, successful: *success }),
        GameEvent::NoPlayersToField { team_id } =>
            Some(WireReport::NoPlayersToField { team_id: team_id.clone() }),
        GameEvent::PlayerNote { player_id, note } =>
            Some(WireReport::PlayerNote { player_id: player_id.clone(), note: note.clone() }),

        // ── Skipped: no Java report/wire counterpart (internal-only or model-change only) ──
        // LonerRoll, ProRoll: internal skill activation rolls with no dedicated ReportId.
        // PlayerMoved, PlayerFellDown: tracked via ModelChange (position), not a report.
        // BallPickedUp, BallScattered: covered by PickupRoll/ScatterBall reports instead.
        // KickoffPitchInvasionStun: folded into the KickoffPitchInvasion report itself.
        // PlayerAdded: uses ServerCommandAddPlayer (a model command), not IReport.
        // PassBlockEligible: internal targeting info; client infers eligibility from model.
        // BuyInducement: individual purchase step; surfaced via CardsAndInducementsBought summary.
        // PlayerEjected, CoachBanned: no dedicated ReportId; surfaced via PlayerNote/PLAYER_EVENT.
        // HeatExhaustion: no dedicated ReportId found; folds into Injury/PlayerNote flow.
        // SpecialEffectRoll: no matching ReportId in report_id.rs.
        event => {
            log::trace!("no wire report for event: {:?}", event);
            None
        }
    }
}

/// Convert a slice of `GameEvent`s to a `Vec<WireReport>`.
pub fn events_to_reports(events: &[GameEvent]) -> Vec<WireReport> {
    events.iter().filter_map(event_to_report).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PassingDistance, PassResult, KickoffResult};
    use ffb_model::enums::ReRollSource;

    fn jsn(report: &WireReport) -> String { serde_json::to_string(report).unwrap() }
    fn sync_jsn(reports: Vec<WireReport>) -> String {
        serde_json::to_string(&OutgoingModelSync::new(1, reports)).unwrap()
    }

    // ── Outgoing wire format ───────────────────────────────────────────────────

    #[test]
    fn model_sync_has_all_required_keys() {
        let json = sync_jsn(vec![]);
        assert!(json.contains("\"netCommandId\":\"serverModelSync\""));
        assert!(json.contains("\"commandNr\":1"));
        assert!(json.contains("\"modelChangeList\""));
        assert!(json.contains("\"modelChangeArray\""));
        assert!(json.contains("\"reportList\""));
        assert!(json.contains("\"reports\""));
        assert!(json.contains("\"animation\":null"));
        assert!(json.contains("\"gameTime\":0"));
        assert!(json.contains("\"turnTime\":0"));
    }

    #[test]
    fn command_nr_is_embedded() {
        let json = serde_json::to_string(&OutgoingModelSync::new(42, vec![])).unwrap();
        assert!(json.contains("\"commandNr\":42"));
    }

    #[test]
    fn report_list_wraps_in_reports_key() {
        let list = WireReportList { reports: vec![WireReport::Block { defender_id: "p1".into() }] };
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.starts_with("{\"reports\":["));
    }

    #[test]
    fn model_change_list_uses_camel_key() {
        let list = WireModelChangeList { model_change_array: vec![] };
        let json = serde_json::to_string(&list).unwrap();
        assert!(json.contains("\"modelChangeArray\""));
    }

    // ── WireReport serialization (report_id tag) ──────────────────────────────

    #[test]
    fn block_report_id_and_fields() {
        let json = jsn(&WireReport::Block { defender_id: "def".into() });
        assert!(json.contains("\"reportId\":\"BLOCK\""));
        assert!(json.contains("\"defenderId\":\"def\""));
    }

    #[test]
    fn block_roll_report_id_and_fields() {
        let json = jsn(&WireReport::BlockRoll {
            attacker_id: "a".into(), defender_id: "d".into(), nr_of_dice: 2,
            dice: vec![1, 3], selected_die: 1, own_choice: true, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"BLOCK_ROLL\""));
        assert!(json.contains("\"nrOfDice\":2"));
        assert!(json.contains("\"ownChoice\":true"));
        assert!(json.contains("\"reRolled\":false"));
    }

    #[test]
    fn dodge_roll_report() {
        let json = jsn(&WireReport::DodgeRoll {
            player_id: "p".into(), successful: true, roll: 4, minimum_roll: 3, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"DODGE_ROLL\""));
        assert!(json.contains("\"successful\":true"));
        assert!(json.contains("\"minimumRoll\":3"));
    }

    #[test]
    fn go_for_it_roll_report() {
        let json = jsn(&WireReport::GoForItRoll {
            player_id: "p".into(), successful: false, roll: 1, minimum_roll: 2, re_rolled: true,
        });
        assert!(json.contains("\"reportId\":\"GO_FOR_IT_ROLL\""));
        assert!(json.contains("\"reRolled\":true"));
    }

    #[test]
    fn pickup_roll_report() {
        let json = jsn(&WireReport::PickupRoll {
            player_id: "p".into(), successful: true, roll: 5, minimum_roll: 3, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"PICKUP_ROLL\""));
    }

    #[test]
    fn catch_roll_report() {
        let json = jsn(&WireReport::CatchRoll {
            player_id: "p".into(), successful: true, roll: 4, minimum_roll: 2, re_rolled: false,
        });
        assert!(json.contains("\"reportId\":\"CATCH_ROLL\""));
    }

    #[test]
    fn pass_roll_report() {
        let json = jsn(&WireReport::PassRoll {
            player_id: "p".into(), successful: true, roll: 3, minimum_roll: 3, re_rolled: false,
            passing_distance: Some("SHORT".into()), pass_result: "ACCURATE".into(),
            hail_mary_pass: false, bomb: false,
        });
        assert!(json.contains("\"reportId\":\"PASS_ROLL\""));
        assert!(json.contains("\"passingDistance\""));
        assert!(json.contains("\"passResult\""));
    }

    #[test]
    fn injury_report_fields() {
        let json = jsn(&WireReport::Injury {
            attacker_id: Some("a".into()), defender_id: "d".into(),
            armor_roll: vec![3, 4], injury_roll: vec![2, 3],
            armor_broken: true, injury: None,
            casualty_roll: vec![], serious_injury: None,
        });
        assert!(json.contains("\"reportId\":\"INJURY\""));
        assert!(json.contains("\"armorRoll\""));
        assert!(json.contains("\"injuryRoll\""));
        assert!(json.contains("\"armorBroken\":true"));
    }

    #[test]
    fn re_roll_report() {
        let json = jsn(&WireReport::ReRoll {
            team_id: "team1".into(), source: "TEAM_RE_ROLL".into(),
            re_rolled_action: Some("DODGE".into()),
        });
        assert!(json.contains("\"reportId\":\"RE_ROLL\""));
        assert!(json.contains("\"teamId\":\"team1\""));
        assert!(json.contains("\"reRolledAction\""));
    }

    #[test]
    fn skill_use_report() {
        let json = jsn(&WireReport::SkillUse {
            player_id: Some("p".into()), skill: "52".into(), used: true, skill_use: "USED".into(),
        });
        assert!(json.contains("\"reportId\":\"SKILL_USE\""));
        assert!(json.contains("\"skillUse\":\"USED\""));
    }

    #[test]
    fn player_action_report() {
        let json = jsn(&WireReport::PlayerAction {
            player_id: "p".into(), player_action: "Move".into(),
        });
        assert!(json.contains("\"reportId\":\"PLAYER_ACTION\""));
        assert!(json.contains("\"playerAction\""));
    }

    #[test]
    fn turn_end_report() {
        let json = jsn(&WireReport::TurnEnd { team_id: "t".into(), turn_nr: 3 });
        assert!(json.contains("\"reportId\":\"TURN_END\""));
        assert!(json.contains("\"turnNr\":3"));
    }

    #[test]
    fn kickoff_result_report() {
        let json = jsn(&WireReport::KickoffResult { result: "BLITZ".into() });
        assert!(json.contains("\"reportId\":\"KICKOFF_RESULT\""));
    }

    #[test]
    fn coin_throw_report_home_won() {
        let json = jsn(&WireReport::CoinThrow { home_won: true });
        assert!(json.contains("\"reportId\":\"COIN_THROW\""));
        assert!(json.contains("\"homeWon\":true"));
    }

    #[test]
    fn receive_choice_report() {
        let json = jsn(&WireReport::ReceiveChoice { team_id: "t".into(), receive: true });
        assert!(json.contains("\"reportId\":\"RECEIVE_CHOICE\""));
        assert!(json.contains("\"receive\":true"));
    }

    #[test]
    fn touchdown_report() {
        let json = jsn(&WireReport::Touchdown { player_id: "scorer".into() });
        assert!(json.contains("\"reportId\":\"TOUCHDOWN\""));
        assert!(json.contains("\"playerId\":\"scorer\""));
    }

    #[test]
    fn foul_report() {
        let json = jsn(&WireReport::Foul { defender_id: "victim".into() });
        assert!(json.contains("\"reportId\":\"FOUL\""));
        assert!(json.contains("\"defenderId\":\"victim\""));
    }

    #[test]
    fn weather_change_report() {
        let json = jsn(&WireReport::WeatherChange { weather: "NICE".into() });
        assert!(json.contains("\"reportId\":\"WEATHER_CHANGE\""));
    }

    // ── event_to_report conversion ────────────────────────────────────────────

    #[test]
    fn event_block_converts() {
        let event = GameEvent::Block { defender_id: "d".into() };
        let report = event_to_report(&event).unwrap();
        let json = jsn(&report);
        assert!(json.contains("\"BLOCK\""));
        assert!(json.contains("\"defenderId\":\"d\""));
    }

    #[test]
    fn event_block_roll_converts() {
        let event = GameEvent::BlockRoll {
            attacker_id: "a".into(), defender_id: "d".into(), nr_of_dice: 1,
            dice: vec![2], selected_index: 0, own_choice: false, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BLOCK_ROLL\""));
        assert!(json.contains("\"attackerId\":\"a\""));
    }

    #[test]
    fn event_dodge_roll_converts() {
        let event = GameEvent::DodgeRoll { player_id: "p".into(), target: 3, roll: 4, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DODGE_ROLL\""));
        assert!(json.contains("\"successful\":true"));
    }

    #[test]
    fn event_go_for_it_converts() {
        let event = GameEvent::GoForItRoll { player_id: "p".into(), target: 2, roll: 1, success: false, rerolled: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"GO_FOR_IT_ROLL\""));
        assert!(json.contains("\"reRolled\":true"));
    }

    #[test]
    fn event_pickup_converts() {
        let event = GameEvent::PickupRoll { player_id: "p".into(), target: 3, roll: 5, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PICKUP_ROLL\""));
    }

    #[test]
    fn event_catch_converts() {
        let event = GameEvent::CatchRoll { player_id: "p".into(), target: 2, roll: 4, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CATCH_ROLL\""));
    }

    #[test]
    fn event_pass_roll_converts() {
        let event = GameEvent::PassRoll {
            player_id: "p".into(), target: 3, distance: PassingDistance::ShortPass,
            roll: 4, result: PassResult::Complete, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PASS_ROLL\""));
        assert!(json.contains("\"successful\":true"));
    }

    #[test]
    fn event_pass_fumble_is_not_successful() {
        let event = GameEvent::PassRoll {
            player_id: "p".into(), target: 3, distance: PassingDistance::ShortPass,
            roll: 1, result: PassResult::Fumble, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"successful\":false"));
    }

    #[test]
    fn event_injury_converts_with_armor_roll() {
        let event = GameEvent::Injury {
            player_id: "hurt".into(),
            armor_roll: Some([3, 4]),
            injury_roll: Some([1, 2]),
            serious_injury: None,
            was_ko: true,
            was_cas: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INJURY\""));
        assert!(json.contains("\"armorBroken\":true"));
        assert!(json.contains("[3,4]"));
    }

    #[test]
    fn event_injury_converts_without_armor_roll() {
        let event = GameEvent::Injury {
            player_id: "p".into(),
            armor_roll: None,
            injury_roll: None,
            serious_injury: None,
            was_ko: false,
            was_cas: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INJURY\""));
        assert!(json.contains("\"armorRoll\":[]"));
    }

    #[test]
    fn event_reroll_converts() {
        let event = GameEvent::ReRoll {
            team_id: "home".into(),
            source: ReRollSource::new("teamReRoll"),
            rerolled_action: "DODGE".into(),
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RE_ROLL\""));
        assert!(json.contains("\"teamId\":\"home\""));
        assert!(json.contains("teamReRoll"));
    }

    #[test]
    fn event_skill_use_converts() {
        let event = GameEvent::SkillUse { player_id: "p".into(), skill_id: 52, used: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SKILL_USE\""));
        assert!(json.contains("\"used\":true"));
        assert!(json.contains("\"skillUse\":\"USED\""));
    }

    #[test]
    fn event_skill_declined_converts() {
        let event = GameEvent::SkillUse { player_id: "p".into(), skill_id: 1, used: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"skillUse\":\"DECLINED\""));
    }

    #[test]
    fn event_player_action_converts() {
        let event = GameEvent::PlayerAction {
            player_id: "p".into(),
            action: ffb_model::enums::PlayerAction::Move,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PLAYER_ACTION\""));
    }

    #[test]
    fn event_turn_end_converts() {
        let event = GameEvent::TurnEnd { team_id: "home".into(), turn_nr: 4 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TURN_END\""));
        assert!(json.contains("\"turnNr\":4"));
    }

    #[test]
    fn event_coin_throw_converts() {
        let event = GameEvent::CoinThrow { home_won: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"COIN_THROW\""));
        assert!(json.contains("\"homeWon\":false"));
    }

    #[test]
    fn event_receive_choice_converts() {
        let event = GameEvent::ReceiveChoice { team_id: "away".into(), receive: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RECEIVE_CHOICE\""));
    }

    #[test]
    fn event_touchdown_converts() {
        let event = GameEvent::Touchdown {
            player_id: "scorer".into(),
            coord: ffb_model::types::FieldCoordinate { x: 0, y: 5 },
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TOUCHDOWN\""));
        assert!(json.contains("\"playerId\":\"scorer\""));
    }

    #[test]
    fn event_foul_converts() {
        let event = GameEvent::Foul {
            attacker_id: "fouler".into(),
            defender_id: "prone".into(),
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"FOUL\""));
        assert!(json.contains("\"defenderId\":\"prone\""));
    }

    #[test]
    fn event_kickoff_result_converts() {
        let event = GameEvent::KickoffResultEvent { result: KickoffResult::Blitz };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_RESULT\""));
    }

    #[test]
    fn unknown_event_returns_none() {
        let event = GameEvent::PlayerMoved {
            player_id: "p".into(),
            coord: ffb_model::types::FieldCoordinate { x: 3, y: 4 },
        };
        assert!(event_to_report(&event).is_none());
    }

    #[test]
    fn events_to_reports_filters_unknowns() {
        let events = vec![
            GameEvent::CoinThrow { home_won: true },
            GameEvent::PlayerMoved {
                player_id: "p".into(),
                coord: ffb_model::types::FieldCoordinate { x: 1, y: 1 },
            },
            GameEvent::Block { defender_id: "d".into() },
        ];
        let reports = events_to_reports(&events);
        assert_eq!(reports.len(), 2);
    }

    // ── Phase ZV: extended coverage tests ─────────────────────────────────────

    #[test]
    fn event_always_hungry_converts() {
        let event = GameEvent::AlwaysHungry { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ALWAYS_HUNGRY_ROLL\""));
        assert!(json.contains("\"successful\":true"));
    }

    #[test]
    fn event_team_captain_roll_converts() {
        let event = GameEvent::TeamCaptainRoll { team_id: "home".into(), roll: 3, reroll_saved: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TEAM_CAPTAIN_ROLL\""));
        assert!(json.contains("\"rerollSaved\":true"));
    }

    #[test]
    fn event_animosity_roll_converts() {
        let event = GameEvent::AnimosityRoll { player_id: "p".into(), roll: 2, success: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ANIMOSITY_ROLL\""));
    }

    #[test]
    fn event_argue_the_call_converts() {
        let event = GameEvent::ArgueTheCall { player_id: "p".into(), roll: 6, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ARGUE_THE_CALL\""));
    }

    #[test]
    fn event_blood_lust_roll_converts() {
        let event = GameEvent::BloodLustRoll { player_id: "p".into(), roll: 1, success: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BLOOD_LUST_ROLL\""));
    }

    #[test]
    fn event_bribes_roll_converts() {
        let event = GameEvent::BribesRoll { player_id: "p".into(), roll: 5, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BRIBES_ROLL\""));
    }

    #[test]
    fn event_baleful_hex_roll_converts() {
        let event = GameEvent::BalefulHexRoll { attacker_id: "a".into(), target_id: "t".into(), roll: 3, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BALEFUL_HEX\""));
        assert!(json.contains("\"targetId\":\"t\""));
    }

    #[test]
    fn event_chainsaw_roll_converts() {
        let event = GameEvent::ChainsawRoll { player_id: "p".into(), roll: 4, minimum_roll: 2, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CHAINSAW_ROLL\""));
        assert!(json.contains("\"minimumRoll\":2"));
    }

    #[test]
    fn event_look_into_my_eyes_roll_converts() {
        let event = GameEvent::LookIntoMyEyesRoll { player_id: "p".into(), roll: 2, success: false, rerolled: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"LOOK_INTO_MY_EYES_ROLL\""));
        assert!(json.contains("\"reRolled\":true"));
    }

    #[test]
    fn event_pick_me_up_roll_converts() {
        let event = GameEvent::PickMeUpRoll { player_id: "p".into(), roll: 3, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PICK_ME_UP\""));
    }

    #[test]
    fn event_confusion_roll_converts() {
        let event = GameEvent::ConfusionRoll { player_id: "p".into(), roll: 2, confused: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CONFUSION_ROLL\""));
        assert!(json.contains("\"confused\":true"));
    }

    #[test]
    fn event_dauntless_roll_converts() {
        let event = GameEvent::DauntlessRoll { player_id: "p".into(), roll: 5, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DAUNTLESS_ROLL\""));
    }

    #[test]
    fn event_escape_roll_converts() {
        let event = GameEvent::EscapeRoll { player_id: "p".into(), roll: 5, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ESCAPE_ROLL\""));
    }

    #[test]
    fn event_foul_appearance_roll_converts() {
        let event = GameEvent::FoulAppearanceRoll { player_id: "p".into(), roll: 1, failed: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"FOUL_APPEARANCE_ROLL\""));
        assert!(json.contains("\"failed\":true"));
    }

    #[test]
    fn event_hypnotic_gaze_roll_converts() {
        let event = GameEvent::HypnoticGazeRoll { player_id: "p".into(), target_id: "t".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"HYPNOTIC_GAZE_ROLL\""));
    }

    #[test]
    fn event_interception_roll_converts() {
        let event = GameEvent::InterceptionRoll { player_id: "p".into(), target: 5, roll: 6, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INTERCEPTION_ROLL\""));
    }

    #[test]
    fn event_jump_roll_converts() {
        let event = GameEvent::JumpRoll { player_id: "p".into(), target: 3, roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"JUMP_ROLL\""));
    }

    #[test]
    fn event_jump_up_roll_converts() {
        let event = GameEvent::JumpUpRoll { player_id: "p".into(), target: 3, roll: 2, success: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"JUMP_UP_ROLL\""));
    }

    #[test]
    fn event_master_chef_roll_converts() {
        let event = GameEvent::MasterChefRoll { team_id: "home".into(), roll: 5, rerolls_stolen: 1 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"MASTER_CHEF_ROLL\""));
        assert!(json.contains("\"rerollsStolen\":1"));
    }

    #[test]
    fn event_piling_on_converts() {
        let event = GameEvent::PilingOn { player_id: "p".into(), target_id: "t".into(), rerolled: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PILING_ON\""));
    }

    #[test]
    fn event_prayer_roll_converts() {
        let event = GameEvent::PrayerRoll { team_id: "home".into(), roll: 3, prayer_id: "blessed".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PRAYER_ROLL\""));
        assert!(json.contains("\"prayerId\":\"blessed\""));
    }

    #[test]
    fn event_regeneration_roll_converts() {
        let event = GameEvent::RegenerationRoll { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"REGENERATION_ROLL\""));
    }

    #[test]
    fn event_right_stuff_roll_converts() {
        let event = GameEvent::RightStuffRoll { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RIGHT_STUFF_ROLL\""));
    }

    #[test]
    fn event_safe_throw_roll_converts() {
        let event = GameEvent::SafeThrowRoll { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SAFE_THROW_ROLL\""));
    }

    #[test]
    fn event_stand_up_roll_converts() {
        let event = GameEvent::StandUpRoll { player_id: "p".into(), target: 4, roll: 5, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"STAND_UP_ROLL\""));
    }

    #[test]
    fn event_swarming_players_roll_converts() {
        let event = GameEvent::SwarmingPlayersRoll { team_id: "away".into(), roll: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SWARMING_PLAYERS_ROLL\""));
    }

    #[test]
    fn event_throw_team_mate_roll_converts() {
        let event = GameEvent::ThrowTeamMateRoll {
            thrower_id: "t1".into(), thrown_id: "t2".into(), roll: 3,
            result: ffb_model::enums::PassResult::Complete,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THROW_TEAM_MATE_ROLL\""));
        assert!(json.contains("\"thrownId\":\"t2\""));
    }

    #[test]
    fn event_weeping_dagger_roll_converts() {
        let event = GameEvent::WeepingDaggerRoll { player_id: "p".into(), roll: 6 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"WEEPING_DAGGER_ROLL\""));
    }

    #[test]
    fn event_spell_effect_roll_converts() {
        let event = GameEvent::SpellEffectRoll { roll: 3 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SPELL_EFFECT_ROLL\""));
    }

    #[test]
    fn event_breathe_fire_roll_converts() {
        let event = GameEvent::BreatheFireRoll {
            attacker_id: "a".into(), defender_id: "d".into(), roll: 4,
            knock_down: true, prone: false, failure: false, rerolled: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BREATHE_FIRE\""));
        assert!(json.contains("\"knockDown\":true"));
    }

    #[test]
    fn event_projectile_vomit_roll_converts() {
        let event = GameEvent::ProjectileVomitRoll { attacker_id: "a".into(), defender_id: "d".into(), roll: 2, success: false, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PROJECTILE_VOMIT\""));
    }

    #[test]
    fn event_trap_door_converts() {
        let event = GameEvent::TrapDoor { player_id: "p".into(), roll: 5, escaped: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TRAP_DOOR\""));
        assert!(json.contains("\"escaped\":true"));
    }

    #[test]
    fn event_pushback_converts() {
        let event = GameEvent::Pushback {
            attacker_id: "a".into(), defender_id: "d".into(),
            squares: vec![ffb_model::types::FieldCoordinate::new(3, 4)],
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PUSHBACK\""));
        assert!(json.contains("\"x\":3"));
    }

    #[test]
    fn event_scatter_ball_converts() {
        let event = GameEvent::ScatterBall { from: ffb_model::types::FieldCoordinate::new(1, 1), directions: vec![1, 2, 3] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SCATTER_BALL\""));
    }

    #[test]
    fn event_scatter_player_converts() {
        let event = GameEvent::ScatterPlayer { player_id: "p".into(), coords: vec![ffb_model::types::FieldCoordinate::new(2, 2)] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SCATTER_PLAYER\""));
    }

    #[test]
    fn event_swoop_player_converts() {
        let event = GameEvent::SwoopPlayer { player_id: "p".into(), coord: ffb_model::types::FieldCoordinate::new(5, 5) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SWOOP_PLAYER\""));
    }

    #[test]
    fn event_throw_in_converts() {
        let event = GameEvent::ThrowIn { coord: ffb_model::types::FieldCoordinate::new(0, 0), direction: 2, distance: 3 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THROW_IN\""));
    }

    #[test]
    fn event_hand_over_converts() {
        let event = GameEvent::HandOver { from_id: "f".into(), to_id: "t".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"HAND_OVER\""));
        assert!(json.contains("\"toId\":\"t\""));
    }

    #[test]
    fn event_kickoff_scatter_converts() {
        let event = GameEvent::KickoffScatter { start: ffb_model::types::FieldCoordinate::new(1, 1), direction: 3, distance: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_SCATTER\""));
    }

    #[test]
    fn event_pass_deviate_converts() {
        let event = GameEvent::PassDeviate { from: ffb_model::types::FieldCoordinate::new(1, 1), scatter_directions: vec![1, 2] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PASS_DEVIATE\""));
    }

    #[test]
    fn event_kickoff_pitch_invasion_converts() {
        let event = GameEvent::KickoffPitchInvasion { home_roll: 4, away_roll: 5 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_PITCH_INVASION\""));
        assert!(json.contains("\"homeRoll\":4"));
    }

    #[test]
    fn event_kickoff_riot_converts() {
        let event = GameEvent::KickoffRiot { turn_modifier: 1, roll: 3 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_RIOT\""));
    }

    #[test]
    fn event_kickoff_throw_a_rock_converts() {
        let event = GameEvent::KickoffThrowARock { player_id: Some("p".into()) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_THROW_A_ROCK\""));
    }

    #[test]
    fn event_riotous_rookies_converts() {
        let event = GameEvent::RiotousRookies { team_id: "home".into(), player_count: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"RIOTOUS_ROOKIES\""));
        assert!(json.contains("\"playerCount\":2"));
    }

    #[test]
    fn event_hit_and_run_converts() {
        let event = GameEvent::HitAndRun { player_id: "p".into(), direction: ffb_model::enums::Direction::North };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"HIT_AND_RUN\""));
    }

    #[test]
    fn event_kick_team_mate_fumble_converts() {
        let event = GameEvent::KickTeamMateFumble;
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICK_TEAM_MATE_FUMBLE\""));
    }

    #[test]
    fn event_prayer_amount_converts() {
        let event = GameEvent::PrayerAmount { tv_home: 100, tv_away: 90, prayer_amount: 10, home_team_receives_prayers: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PRAYER_AMOUNT\""));
        assert!(json.contains("\"homeTeamReceivesPrayers\":false"));
    }

    #[test]
    fn event_bite_spectator_converts() {
        let event = GameEvent::BiteSpectator { player_id: "p".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BITE_SPECTATOR\""));
    }

    #[test]
    fn event_cards_and_inducements_bought_converts() {
        let event = GameEvent::CardsAndInducementsBought {
            team_id: "home".into(), cards: 1, inducements: 2, stars: 0, mercenaries: 1, gold: 50000, new_tv: 1100000,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CARDS_AND_INDUCEMENTS_BOUGHT\""));
        assert!(json.contains("\"newTv\":1100000"));
    }

    #[test]
    fn event_kickoff_sequence_activations_exhausted_converts() {
        let event = GameEvent::KickoffSequenceActivationsExhausted { limit_reached: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_SEQUENCE_ACTIVATIONS_EXHAUSTED\""));
    }

    #[test]
    fn event_solid_defence_roll_converts() {
        let event = GameEvent::SolidDefenceRoll { team_id: "home".into(), roll: 5, amount: 3 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SOLID_DEFENCE_ROLL\""));
    }

    #[test]
    fn event_cheering_fans_converts() {
        let event = GameEvent::CheeringFans { home_roll: 4, away_roll: 6 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_CHEERING_FANS\""));
    }

    #[test]
    fn event_kickoff_extra_re_roll_converts() {
        let event = GameEvent::KickoffExtraReRoll { team_id: "home".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_EXTRA_RE_ROLL\""));
    }

    #[test]
    fn event_quick_snap_roll_converts() {
        let event = GameEvent::QuickSnapRoll { team_id: "home".into(), roll: 4, amount: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"QUICK_SNAP_ROLL\""));
    }

    #[test]
    fn event_kickoff_timeout_converts() {
        let event = GameEvent::KickoffTimeout { turn_number: 4, turn_modifier: 1 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_TIMEOUT\""));
    }

    #[test]
    fn event_kickoff_officious_ref_converts() {
        let event = GameEvent::KickoffOfficiousRef { roll_home: 3, roll_away: 4, player_ids: vec!["p1".into()] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_OFFICIOUS_REF\""));
    }

    #[test]
    fn event_kickoff_dodgy_snack_converts() {
        let event = GameEvent::KickoffDodgySnack { roll_home: 2, roll_away: 5, player_ids: vec!["p1".into()] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_DODGY_SNACK\""));
    }

    #[test]
    fn event_dodgy_snack_roll_converts() {
        let event = GameEvent::DodgySnackRoll { player_id: "p".into(), roll: 3 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DODGY_SNACK_ROLL\""));
    }

    #[test]
    fn event_throw_at_player_converts() {
        let event = GameEvent::ThrowAtPlayer { player_id: "p".into(), roll: 4, successful: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THROW_AT_PLAYER\""));
    }

    #[test]
    fn event_fumblerooskie_converts() {
        let event = GameEvent::Fumblerooskie { player_id: "p".into(), used: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"FUMBLEROOSKIE\""));
    }

    #[test]
    fn event_all_you_can_eat_roll_converts() {
        let event = GameEvent::AllYouCanEatRoll { player_id: "p".into(), roll: 4, minimum_roll: 2, success: true, rerolled: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ALL_YOU_CAN_EAT\""));
    }

    #[test]
    fn event_kickoff_extra_re_roll_bb2016_converts() {
        let event = GameEvent::KickoffExtraReRollBb2016 {
            kickoff_result: ffb_model::enums::KickoffResult::CheeringFans,
            roll_home: 3, home_gains_reroll: true, roll_away: 2, away_gains_reroll: false,
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_EXTRA_RE_ROLL\""));
        assert!(json.contains("\"homeGainsReroll\":true"));
    }

    #[test]
    fn event_kickoff_throw_a_rock_bb2016_converts() {
        let event = GameEvent::KickoffThrowARockBb2016 { roll_home: 2, roll_away: 3, player_ids: vec!["p1".into(), "p2".into()] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_THROW_A_ROCK\""));
    }

    #[test]
    fn event_kickoff_pitch_invasion_bb2016_converts() {
        let event = GameEvent::KickoffPitchInvasionBb2016 {
            rolls_home: vec![1, 2], affected_home: vec![false, true],
            rolls_away: vec![3, 4], affected_away: vec![true, false],
        };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"KICKOFF_PITCH_INVASION\""));
        assert!(json.contains("\"affectedHome\":[false,true]"));
    }

    #[test]
    fn event_animal_savagery_converts() {
        let event = GameEvent::AnimalSavagery { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"ANIMAL_SAVAGERY\""));
    }

    #[test]
    fn event_leader_converts() {
        let event = GameEvent::Leader { player_id: "p".into(), reroll_available: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"LEADER\""));
        assert!(json.contains("\"rerollAvailable\":true"));
    }

    #[test]
    fn event_then_i_started_blastin_converts() {
        let event = GameEvent::ThenIStartedBlastin { attacker_id: "a".into(), defender_id: Some("d".into()), roll: 4, success: true, fumble: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THEN_I_STARTED_BLASTIN\""));
    }

    #[test]
    fn event_apothecary_roll_converts() {
        let event = GameEvent::ApothecaryRoll { player_id: "p".into(), roll: Some(4), new_state: Some(1), new_serious_injury: None };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"APOTHECARY_ROLL\""));
        assert!(json.contains("\"newState\":1"));
    }

    #[test]
    fn event_apothecary_choice_converts() {
        let event = GameEvent::ApothecaryChoice { player_id: "p".into(), healed: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"APOTHECARY_CHOICE\""));
    }

    #[test]
    fn event_select_blitz_target_converts() {
        let event = GameEvent::SelectBlitzTarget { attacker_id: "a".into(), defender_id: "d".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SELECT_BLITZ_TARGET\""));
    }

    #[test]
    fn event_select_gaze_target_converts() {
        let event = GameEvent::SelectGazeTarget { attacker_id: "a".into(), defender_id: "d".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SELECT_GAZE_TARGET\""));
    }

    #[test]
    fn event_card_deactivated_converts() {
        let event = GameEvent::CardDeactivated { card_id: "c1".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CARD_DEACTIVATED\""));
    }

    #[test]
    fn event_card_effect_roll_converts() {
        let event = GameEvent::CardEffectRoll { card_id: "c1".into(), roll: 3, effect: "boost".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"CARD_EFFECT_ROLL\""));
    }

    #[test]
    fn event_defecting_players_converts() {
        let event = GameEvent::DefectingPlayers { player_ids: vec!["p1".into(), "p2".into()] };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DEFECTING_PLAYERS\""));
    }

    #[test]
    fn event_pass_block_converts() {
        let event = GameEvent::PassBlock { player_id: Some("p".into()) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PASS_BLOCK\""));
    }

    #[test]
    fn event_petty_cash_converts() {
        let event = GameEvent::PettyCash { team_id: "home".into(), amount: 20000 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PETTY_CASH\""));
    }

    #[test]
    fn event_play_card_converts() {
        let event = GameEvent::PlayCard { team_id: "home".into(), card_id: "c1".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PLAY_CARD\""));
    }

    #[test]
    fn event_secret_weapon_ban_converts() {
        let event = GameEvent::SecretWeaponBan { player_id: "p".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"SECRET_WEAPON_BAN\""));
    }

    #[test]
    fn event_wizard_use_converts() {
        let event = GameEvent::WizardUse { team_id: "home".into(), spell: "fireball".into(), coord: Some(ffb_model::types::FieldCoordinate::new(1, 1)) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"WIZARD_USE\""));
    }

    #[test]
    fn event_bomb_explodes_after_catch_converts() {
        let event = GameEvent::BombExplodesAfterCatch { player_id: "p".into(), coord: ffb_model::types::FieldCoordinate::new(2, 2) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BOMB_EXPLODES_AFTER_CATCH\""));
    }

    #[test]
    fn event_bomb_out_of_bounds_converts() {
        let event = GameEvent::BombOutOfBounds { coord: ffb_model::types::FieldCoordinate::new(0, 0) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BOMB_OUT_OF_BOUNDS\""));
    }

    #[test]
    fn event_keg_throw_converts() {
        let event = GameEvent::KegThrow { thrower_id: "t".into(), target_id: Some("v".into()), roll: 4, success: true, fumble: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THROWN_KEG\""));
    }

    #[test]
    fn event_inducement_converts() {
        let event = GameEvent::Inducement { team_id: "home".into(), inducement_type: "wandering_apo".into(), value: 1 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"INDUCEMENT\""));
    }

    #[test]
    fn event_pump_up_the_crowd_re_roll_converts() {
        let event = GameEvent::PumpUpTheCrowdReRoll { player_id: "p".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PUMP_UP_THE_CROWD_RE_ROLL\""));
    }

    #[test]
    fn event_referee_spots_foul_converts() {
        let event = GameEvent::RefereeSpotsFoul { referee_spots_foul: true, under_scrutiny: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"REFEREE\""));
        assert!(json.contains("\"refereeSpotsFoul\":true"));
    }

    #[test]
    fn event_biased_ref_roll_converts() {
        let event = GameEvent::BiasedRefRoll { roll: 3, referee_spots_foul: false };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BIASED_REF\""));
    }

    #[test]
    fn event_double_hired_star_player_converts() {
        let event = GameEvent::DoubleHiredStarPlayer;
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"DOUBLE_HIRED_STAR_PLAYER\""));
    }

    #[test]
    fn event_game_options_converts() {
        let mut options = std::collections::HashMap::new();
        options.insert("kickTeam".to_string(), "home".to_string());
        let event = GameEvent::GameOptions { options_snapshot: options };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"GAME_OPTIONS\""));
    }

    #[test]
    fn event_start_half_converts() {
        let event = GameEvent::StartHalf { half: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"START_HALF\""));
        assert!(json.contains("\"half\":2"));
    }

    #[test]
    fn event_timeout_enforced_converts() {
        let event = GameEvent::TimeoutEnforced { team_id: "home".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"TIMEOUT_ENFORCED\""));
    }

    #[test]
    fn event_winnings_roll_converts() {
        let event = GameEvent::WinningsRoll { team_id: "home".into(), base: 10000, roll: 4, total: 40000 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"WINNINGS_ROLL\""));
        assert!(json.contains("\"total\":40000"));
    }

    #[test]
    fn event_mvp_roll_converts() {
        let event = GameEvent::MvpRoll { team_id: "home".into(), player_id: "p".into(), spp: 6 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"MOST_VALUABLE_PLAYERS\""));
        assert!(json.contains("\"spp\":6"));
    }

    #[test]
    fn event_blitz_roll_converts() {
        let event = GameEvent::BlitzRoll { team_id: "home".into(), roll: 4, limit: 2 };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"BLITZ_ROLL\""));
    }

    #[test]
    fn event_throw_at_stalling_player_converts() {
        let event = GameEvent::ThrowAtStallingPlayer { player_id: "p".into(), roll: 4, success: true };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"THROW_AT_STALLING_PLAYER\""));
    }

    #[test]
    fn event_no_players_to_field_converts() {
        let event = GameEvent::NoPlayersToField { team_id: Some("away".into()) };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"NO_PLAYERS_TO_FIELD\""));
    }

    #[test]
    fn event_player_note_converts() {
        let event = GameEvent::PlayerNote { player_id: "p".into(), note: "is stalling".into() };
        let json = jsn(&event_to_report(&event).unwrap());
        assert!(json.contains("\"PLAYER_EVENT\""));
        assert!(json.contains("\"note\":\"is stalling\""));
    }
}
