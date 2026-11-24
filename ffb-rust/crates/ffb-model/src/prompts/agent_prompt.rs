use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerAction, ReRollSource, SkillCategory, SkillId, Weather};
use crate::model::player::PlayerId;
use crate::types::FieldCoordinate;

fn default_true() -> bool { true }

/// The server's request for an agent decision (replaces 70+ Java Dialog*Parameter classes).
///
/// The engine emits one of these whenever it needs an agent choice.
/// The agent calls `apply()` with the matching `AgentResponse`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentPrompt {
    /// Java: bb2016 `StepInitBlocking` with no `blockDefenderId` yet falls through `executeStep`
    /// without calling `setNextAction`, so the step keeps CONTINUE and waits for a `CLIENT_BLOCK`.
    /// Rust picks its block target at ACTIVATION time, so a block sequence the ENGINE pushes
    /// (`StepEndBlocking`'s `isSufferingBloodLust() && !hasBlocked()` branch re-pushes one with NO
    /// defender) had nobody to ask and stalled. This prompt is that ask — the agent answers it
    /// exactly as `ParityRunner.sendBlockAction` answers Java's.
    BlockTarget {
        attacker_id: PlayerId,
    },
    /// Java: `StepSelectBlitzTarget` CONTINUEs on a `DialogSelectBlitzTargetParameter`, waiting
    /// for `CLIENT_TARGET_SELECTED`. Distinct from `BlockTarget`, which is the mid-sequence
    /// "no defender" ask that both agents answer by deselecting - this one carries the real
    /// candidate list and expects a pick.
    ///
    /// Candidates and order match `ParityRunner.pickBlockTarget`: ADJACENT opponents whose state
    /// base is STANDING or MOVING (NOT hasTackleZones, no confused check), coordinate-sorted,
    /// answered with exactly ONE actionRng draw.
    BlitzTarget {
        attacker_id: PlayerId,
        eligible_players: Vec<PlayerId>,
        /// The attacker's unused blitz-declaration skill (Frenzied Rush `canGainFrenzyForBlitz`,
        /// Slashing Nails `canGainClawsForBlitz`) while the step has no `usedSkill` yet: the
        /// client offers it from this dialog as a CLIENT_USE_SKILL sent before the target.
        #[serde(default)]
        gain_skill: Option<crate::enums::SkillId>,
    },
    // ── Block ──────────────────────────────────────────────────────────────────
    BlockChoice {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        dice: Vec<i32>,
        own_choice: bool,
        nr_of_dice: i32,
    },
    /// The SKILL half of Java's block-roll dialog: asked after its TRR half (`ReRollOffer`
    /// action BLOCK) and before the die choice, only while the dialog lists a Brawler
    /// (SINGLE_BOTH_DOWN / bb2020 `brawlerOption`) or Pro (SINGLE_DIE_PER_ACTIVATION /
    /// bb2020 `proReRollOption`) re-roll. Answered with `Action::UseBrawler`,
    /// `Action::UseProReRollForBlock` or, to decline, `Action::UseReRoll { use_reroll: false }`
    /// -- always with ZERO sampler draws.
    BlockSkillReRoll {
        attacker_id: PlayerId,
        dice: Vec<i32>,
        own_choice: bool,
        brawler: bool,
        pro: bool,
        /// The dialog's SINGLE_BLOCK_DIE skill source, by Java `ReRollSource` name (bb2025
        /// `actionToSource[SINGLE_BLOCK_DIE]`: Woodland Fury / Unstoppable Momentum / Lord of
        /// Chaos / Working in Tandem; bb2020 `singleBlockDieOption`: Unstoppable Momentum).
        /// Answered with `Action::UseSingleBlockDieReRoll` (BACKLOG §M).
        #[serde(default)]
        single_block_die: Option<String>,
        /// The dialog lists a MULTI_BLOCK_DICE skill (Savage Blow; bb2020 `anyBlockDiceOption`).
        /// Answered with `Action::UseMultiBlockDiceReRoll` (BACKLOG §M).
        #[serde(default)]
        multi_block_dice: bool,
    },
    /// bb2020 `StepSelectGazeTarget`'s DialogSelectGazeTarget (TurnMode SELECT_GAZE_TARGET):
    /// answered with `Action::SelectPlayer` naming the victim, or the attacker himself to cancel.
    /// The agents answer with the victim they drew when the gaze was declared -- the same single
    /// actionRng pick `ParityRunner.sendGazeTargetSelection` spends here (BACKLOG §K).
    GazeTarget {
        attacker_id: PlayerId,
        /// §M: the acting player's unused bb2020 Zoat skill (canGainGaze) when the step has none
        /// yet -- the GAZE_ZOAT client sends UseSkill(it) before naming the victim.
        #[serde(default)]
        gain_skill: Option<crate::enums::SkillId>,
    },
    BlockChoiceProperties {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        can_reroll: bool,
        reroll_sources: Vec<ReRollSource>,
    },

    // ── Movement ───────────────────────────────────────────────────────────────
    ActivatePlayer {
        eligible_players: Vec<(PlayerId, Vec<PlayerAction>)>,
    },
    Move {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    /// Throw Team-Mate: the thrower has picked up `thrown_player_id` (now PICKED_UP) and must choose
    /// the target square to throw it to. Emitted by StepInitThrowTeamMate after the pick-up.
    ThrowTeamMateTarget {
        thrower_id: PlayerId,
        thrown_player_id: PlayerId,
    },
    /// Punt (BB2025): the punter must choose a target square (Java StepInitPunt waits for
    /// CLIENT_PUNT / CLIENT_END_TURN with the punt squares as MoveSquares). The parity
    /// harness has no INIT_PUNT handler and always aborts with EndTurn.
    PuntTarget {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    /// Swoop (BB2016/BB2020): a thrown player with `ttmScattersInSingleDirection` must be steered
    /// into one of the (at most four) orthogonally adjacent squares — Java's mixed `StepSwoop` ends
    /// its execute with `UtilServerPlayerSwoop.updateSwoopSquares` and then WAITS for a
    /// `CLIENT_SWOOP` naming the square. Unlike BB2025, where Swoop is an optional skill offer the
    /// harness declines, the BB2016/BB2020 step has no decline path: something must answer it or the
    /// step never advances.
    SwoopTarget {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    FollowUp {
        attacker_id: PlayerId,
        target_coord: FieldCoordinate,
    },
    HitAndRun {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    /// Raiding Party target-square window (Java: MoveSquares published + CLIENT_FIELD_COORDINATE
    /// wait, turn mode RAIDING_PARTY). Same answer contract as PuntTarget/HitAndRun:
    /// coordinate-sorted squares, single actionRng pick.
    /// Java: StepFirstMove/SecondMoveFuriousOutburst publish their eligible squares as
    /// MoveSquares and wait for a `ClientCommandFieldCoordinate` naming one. No dialog.
    FuriousOutburstSquare {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    RaidingParty {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    /// "Blastin' Solves Everything" target wait (Java: TurnMode THEN_I_STARTED_BLASTIN, no
    /// dialog — the client answers with CLIENT_TARGET_SELECTED). Fired at BOTH waits: the
    /// initial pick (source = the acting star, opponents only) and the roll-2 replacement
    /// pick (source = the original target, either team, acting player excluded — the
    /// opposing coach chooses). `candidates` comes from the STEP per the client's
    /// isValidTarget; answer = coordinate-sorted single actionRng pick → Action::SelectPlayer,
    /// or EndTurn when empty.
    BlastinTarget {
        player_id: PlayerId,
        candidates: Vec<PlayerId>,
    },
    TricksterMove {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    /// Safe Pair of Hands square wait (BACKLOG §M; Java bb2020/bb2025 StepPlaceBall Phase.SELECT:
    /// the free squares adjacent to the ball published as MoveSquares, TurnMode
    /// SAFE_PAIR_OF_HANDS, waiting for CLIENT_FIELD_COORDINATE from the carrier's coach).
    /// Answer: `Action::PlaceBallSquare`, coordinate-sorted, single actionRng pick.
    PlaceBallSquare {
        player_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },
    Pushback {
        attacker_id: PlayerId,
        defender_id: PlayerId,
        squares: Vec<FieldCoordinate>,
    },

    // ── Re-rolls ───────────────────────────────────────────────────────────────
    ReRollOffer {
        source: ReRollSource,
        action: String,
        team_id: String,
        /// Java dialog `getReRollSkill()`, kept only when it is a `canRerollSingleDieOncePerPeriod`
        /// skill (Halfling Luck, Thinking Man's Troll, bb2020 Consummate Professional) -- the only
        /// kind `AbstractStepWithReRoll.handleCommand`'s CLIENT_USE_SKILL arm accepts. Answered
        /// with `Action::UseSkillReRoll` (BACKLOG §M phase 2b(a)).
        #[serde(default)]
        re_roll_skill: Option<SkillId>,
        /// The `teamReRollOption` the parity harness reads: bb2016/bb2020 pass the dialog's real
        /// `isTeamReRollOption()`, bb2025 (`DialogReRollPropertiesParameter`) a hardcoded `true`.
        /// `false` = the harness declines with zero draws unless it uses `re_roll_skill`.
        #[serde(default = "default_true")]
        team_re_roll_option: bool,
    },
    /// Java: `DialogReRollForTargetsParameter` (via `AbstractStepModifierMultipleBlock.
    /// decideNextStep`/`createDialogParameter`) — lets the coach pick which still-failing
    /// multi-block target to re-roll, and with what source, for Dauntless-multi/
    /// FoulAppearance-multi.
    ReRollForTargets {
        player_id: PlayerId,
        target_ids: Vec<String>,
        minimum_rolls: HashMap<String, i32>,
        re_rolled_action: String,
        re_roll_available_against: Vec<String>,
        pro_re_roll_available: bool,
        team_re_roll_available: bool,
        consummate_available: bool,
        re_roll_skill: Option<SkillId>,
        single_use_re_roll_source: Option<ReRollSource>,
    },

    // ── Skill use ──────────────────────────────────────────────────────────────
    SkillUse {
        player_id: PlayerId,
        skill_id: u16,
        skill_name: String,
    },
    PilingOn {
        player_id: PlayerId,
        target_id: PlayerId,
    },
    DefenderAction {
        player_id: PlayerId,
        actions: Vec<String>,
    },

    // ── Interception / catch ───────────────────────────────────────────────────
    Interception {
        /// The first candidate — kept for the wire prompt and for existing consumers.
        player_id: PlayerId,
        target_number: i32,
        /// Every player `UtilPassing.findInterceptors` returned, in the engine's own order.
        /// Java's `DialogInterceptionParameter` carries only the thrower, so its client (and
        /// `ParityRunner`) recomputes the same list from the same shared util — publishing it here
        /// lets both harnesses pick from an identical candidate set.
        candidates: Vec<PlayerId>,
        /// Java `DialogInterceptionParameter.getInterceptionSkill()`: bb2020/bb2025
        /// `StepIntercept` :144-149 puts the first candidate's unused `canInterceptEasily` skill
        /// (Yoink!) here; the client offers "Use <skill>" and answers
        /// `sendInterceptorChoice(player, skill)` (`DialogInterceptionHandler`).
        #[serde(default)]
        interception_skill: Option<SkillId>,
    },

    // ── Apothecary ─────────────────────────────────────────────────────────────
    ApothecaryChoice {
        player_id: PlayerId,
        can_heal: bool,
    },
    UseApothecary {
        player_id: PlayerId,
        apothecary_type: String,
    },
    /// Java `DialogUseApothecariesParameter(teamId, injuryDescriptions)` (multi-block casualties):
    /// answered per player with `Action::UseApothecary` (CLIENT_USE_APOTHECARY) or, to decline
    /// them all, `Action::Acknowledge` (CLIENT_USE_APOTHECARIES).
    UseApothecaries {
        team_id: String,
        player_ids: Vec<PlayerId>,
    },

    // ── Team setup ─────────────────────────────────────────────────────────────
    TeamSetup {
        team_id: String,
        players: Vec<PlayerId>,
    },
    SetupError {
        team_id: String,
        error: String,
    },

    // ── Kickoff / game flow ────────────────────────────────────────────────────
    CoinChoice { is_home: bool },
    ReceiveChoice { team_id: String },
    Touchback { eligible_players: Vec<(PlayerId, FieldCoordinate)> },
    KickoffReturn { eligible_players: Vec<PlayerId> },
    KickBall,
    /// An interactive kickoff event (Quick Snap / Solid Defence / High Kick) is waiting for
    /// the coach to optionally move players. Java models this purely via TurnMode (no dialog);
    /// ParityRunner answers with EndTurn at APPLY_KICKOFF_RESULT, declining the placements.
    /// `mode` is the TurnMode's name (e.g. "QuickSnap").
    KickoffEventPlacement { team_id: String, mode: String },
    /// The bomb re-throw window (TurnMode BombHome / BombAway). A player who catches a thrown
    /// bomb becomes the acting player with THROW_BOMB and `StepInitPassing` PARKS with no
    /// thrower, waiting for a client command naming the new target. Java models this with no
    /// dialog at all -- the step simply waits -- so this prompt exists only to surface that wait
    /// to the agent, exactly as `KickoffEventPlacement` does. ParityRunner answers it at
    /// INIT_PASSING with EndTurn (declining the re-throw), and the Rust agent mirrors that.
    BombRethrow { player_id: PlayerId },
    /// The Dump-Off pass window (TurnMode DUMP_OFF, BACKLOG §M): StepDumpOff showed Java's
    /// `DialogDefenderActionParameter` and pushed the Pass sequence; StepInitPassing PARKS
    /// (the stale pass coordinate is out of range) until the DEFENDING coach sends CLIENT_PASS.
    /// Answer: `Action::Pass` at one of `legal_actions::dump_off_receivers`, or an empty
    /// `ActivatePlayer` (CLIENT_ACTING_PLAYER null) when there is none.
    DumpOffTarget { thrower_id: PlayerId },

    /// Java `DialogKickSkillParameter` (all three editions' StepKickoffScatterRoll): the kicker
    /// has Kick (canReduceKickDistance) and stands in the centre field. Answered with
    /// `Action::UseSkill { skill_id: Kick, use_skill }`; both parity agents and ParityRunner's
    /// KICK_SKILL arm DECLINE with zero RNG (the kick choice changes the scatter dice in bb2025).
    KickSkill { player_id: PlayerId },

    /// BACKLOG §M 2b(e): the blitz block of a blitzer holding an unused `canAddBlockDie` skill
    /// (Gored by the Bull). Java's client answers the blitz's second pass with either
    /// CLIENT_BLOCK or CLIENT_USE_SKILL(skill) (BlitzLogicModule); Rust folds that pass into
    /// `StepInitSelecting::start`, so the choice surfaces as this prompt -- raised ONLY for such
    /// a blitzer. Answer `Action::Block { defender_id }` (the plain block) or
    /// `Action::UseSkill { skill_id, use_skill: true }` (StepInitSelecting's canAddBlockDie arm).
    BlitzBlockSkill { player_id: PlayerId, defender_id: PlayerId, skill_id: SkillId },

    // ── Inducements ────────────────────────────────────────────────────────────
    /// bb2016 `StepBuyCards` / `DialogBuyCardsParameter`: the team may buy one card at a time
    /// with `available_gold`. Answered with `Action::BuyCard { card_type }`; `None` = done.
    BuyCards {
        team_id: String,
        available_gold: i32,
    },
    BuyInducements {
        team_id: String,
        available: Vec<(String, i32)>,
        budget: i32,
    },
    BuyPrayersAndInducements {
        team_id: String,
        available: Vec<(String, i32)>,
        prayers: Vec<(String, i32)>,
        budget: i32,
    },
    PettyCash {
        team_id: String,
        amount: i32,
    },
    UseInducement {
        team_id: String,
        inducement_id: String,
    },
    WizardSpell {
        team_id: String,
        target_coord: Option<FieldCoordinate>,
    },
    Journeymen {
        team_id: String,
        count: i32,
    },

    /// Multiple Block target selection (bb2020/bb2025): the acting player declared
    /// MULTIPLE_BLOCK and must now pick TWO adjacent blockable opponents — the client sends
    /// CLIENT_SYNCHRONOUS_MULTI_BLOCK with both targets. `eligible_players` is
    /// coordinate-sorted; the agents draw two actionRng picks (idx % N, then idx % (N-1)).
    MultiBlockTargets {
        player_id: PlayerId,
        eligible_players: Vec<PlayerId>,
    },

    // ── Misc ───────────────────────────────────────────────────────────────────
    PlayerChoice {
        eligible_players: Vec<PlayerId>,
        reason: String,
        /// Java: `DialogPlayerChoiceParameter.descriptions` — a flat list of explanatory
        /// tooltip strings for the dialog as a whole (NOT parallel/indexed to
        /// `eligible_players` — Java's own call sites pass 0 or 1 entries regardless of how
        /// many eligible players there are). Empty when the Java call site passes `null`.
        descriptions: Vec<String>,
    },
    SelectPosition {
        available_positions: Vec<String>,
    },
    /// Java: `DialogSelectSkillParameter(playerId, List<Skill> skills, SkillChoiceMode)` —
    /// choose ONE skill from a flat list. Shared by two mechanics: the Intensive Training prayer
    /// (`SkillChoiceMode.INTENSIVE_TRAINING`) and Wisdom of the White Dwarf
    /// (`WISDOM_OF_THE_WHITE_DWARF`). Answered by `ClientCommandSkillSelection` (Java reuses the
    /// `CLIENT_PRAYER_SELECTION` command id for both).
    ///
    /// `skill_ids` arrives in JAVA'S OWN ORDER, which both call sites sort by skill NAME
    /// (`Comparator.comparing(Skill::getName)`), so index 0 is a genuine contract.
    ///
    /// This used to be `available: Vec<(SkillCategory, Vec<u16>)>` — a Rust-only category
    /// grouping with no Java counterpart, which `step_prayer` had to build ("the prompt groups
    /// them by category because that is the shape AgentPrompt::SelectSkill takes") and which both
    /// agents immediately flattened and re-sorted. Java's dialog carries a flat list.
    SelectSkill {
        player_id: PlayerId,
        skill_ids: Vec<u16>,
        /// Java `SkillChoiceMode` name, e.g. "INTENSIVE_TRAINING".
        reason: String,
    },
    SelectWeather {
        options: Vec<Weather>,
    },
    /// Java `DialogArgueTheCallParameter(teamId, ..., playerIds)`: the team's coach names the
    /// players to argue for (CLIENT_ARGUE_THE_CALL, `Action::ArgueTheCall`); an empty list declines.
    ArgueTheCall {
        team_id: String,
        player_ids: Vec<PlayerId>,
    },
    /// Java `DialogBribesParameter(teamId, maxNrOfBribes, playerIds)`: use a bribe (the team's
    /// AVOID_BAN `inducement_type`) for up to `max_nr_of_bribes` of `player_ids`, answered with
    /// CLIENT_USE_INDUCEMENT (`Action::UseInducement`); an empty player list declines.
    Bribes {
        team_id: String,
        inducement_type: String,
        player_ids: Vec<PlayerId>,
        max_nr_of_bribes: i32,
    },
    BriberyAndCorruption {
        team_id: String,
    },
    /// Java `DialogKickOffResultParameter(teamId)`: in overtime with OVERTIME_KICK_OFF_RESULTS =
    /// blitzOrSolidDefence the kicking coach picks the kick-off result
    /// (CLIENT_KICK_OFF_RESULT_CHOICE, `Action::KickOffResultChoice`).
    KickOffResult {
        team_id: String,
    },
    /// Java `DialogPuntToCrowdParameter`: a punter on a sideline or in an end zone may punt into the
    /// crowd (CLIENT_PUNT_TO_CROWD, `Action::PuntToCrowd`).
    PuntToCrowd {
        player_id: PlayerId,
    },
    /// Java `DialogPickUpChoiceParameter` (bb2025 StepTrickster): the defender landed on the
    /// loose ball -- try to pick it up or let it scatter (CLIENT_PICK_UP_CHOICE,
    /// `Action::PickUpChoice`).
    PickUpChoice {
        player_id: PlayerId,
    },
    /// Java StepBlockRollMultiple's die choice for a Multiple Block: the attacker's
    /// DialogReRollBlockForTargets(Properties)Parameter (`attacker`) or the defending coach's
    /// DialogOpponentBlockSelection(Properties)Parameter. `targets` = (target id, still needs a die).
    /// Answered per target with `Action::BlockChoice { die_index, target_id }` or, from the
    /// client, CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET (`Action::BlockOrReRollChoiceForTarget`).
    BlockRollsForTargets {
        team_id: String,
        attacker: bool,
        targets: Vec<(PlayerId, bool)>,
    },
    ConcedeGame {
        team_id: String,
    },
    ConfirmEndAction {
        team_id: String,
    },
    InformationOkay {
        message: String,
    },
    /// Java: DialogBloodlustActionParameter — vampire failed blood lust; offer action change.
    BloodlustAction {
        player_id: PlayerId,
    },
    SwarmingPlayers {
        team_id: String,
        eligible_players: Vec<PlayerId>,
    },
    StartGame,
    GameStatistics,
}

/// An agent's response to a prompt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentResponse {
    BlockChoice { index: i32 },
    UseReRoll { use_reroll: bool },
    UseSkill { use_skill: bool },
    FollowUp { follow_up: bool },
    Pushback { coord: FieldCoordinate },
    ActivatePlayer { player_id: PlayerId, action: PlayerAction },
    ReceiveChoice { receive: bool },
    CoinChoice { heads: bool },
    Touchback { player_id: PlayerId },
    KickBall { coord: FieldCoordinate },
    PlayerChoice { player_id: PlayerId },
    TeamSetup { placements: Vec<(PlayerId, FieldCoordinate)> },
    ApothecaryChoice { heal: bool },
    UseBribe { use_bribe: bool },
    BuyInducements { purchases: Vec<(String, i32)> },
    SelectSkill { skill_id: u16 },
    Confirm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_coin_choice() {
        let p = AgentPrompt::CoinChoice { is_home: true };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_agent_response() {
        let r = AgentResponse::UseReRoll { use_reroll: true };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn serde_block_choice_prompt() {
        let p = AgentPrompt::BlockChoice {
            attacker_id: "att".into(), defender_id: "def".into(),
            dice: vec![1, 3, 5], own_choice: true, nr_of_dice: 2,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_select_skill_prompt() {
        let p = AgentPrompt::SelectSkill {
            player_id: "p1".into(),
            skill_ids: vec![1, 2, 3],
            reason: "INTENSIVE_TRAINING".into(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_pushback_prompt() {
        let p = AgentPrompt::Pushback {
            attacker_id: "att".into(), defender_id: "def".into(),
            squares: vec![
                FieldCoordinate::new(10, 7),
                FieldCoordinate::new(10, 8),
            ],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_trickster_move_prompt() {
        let p = AgentPrompt::TricksterMove {
            player_id: "trickster".into(),
            squares: vec![FieldCoordinate::new(5, 5)],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_activate_player_prompt() {
        use crate::enums::PlayerAction;
        let p = AgentPrompt::ActivatePlayer {
            eligible_players: vec![
                ("p1".into(), vec![PlayerAction::Move, PlayerAction::Block]),
                ("p2".into(), vec![PlayerAction::Move]),
            ],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn agent_response_select_skill_round_trip() {
        let r = AgentResponse::SelectSkill { skill_id: 42 };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn agent_response_team_setup_round_trip() {
        let r = AgentResponse::TeamSetup {
            placements: vec![("p1".into(), FieldCoordinate::new(5, 7))],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn serde_reroll_for_targets_prompt() {
        let mut minimum_rolls = HashMap::new();
        minimum_rolls.insert("t1".to_string(), 4);
        let p = AgentPrompt::ReRollForTargets {
            player_id: "p1".into(),
            target_ids: vec!["t1".into()],
            minimum_rolls,
            re_rolled_action: "DAUNTLESS".into(),
            re_roll_available_against: vec!["t1".into()],
            pro_re_roll_available: false,
            team_re_roll_available: true,
            consummate_available: false,
            re_roll_skill: None,
            single_use_re_roll_source: Some(ReRollSource::new("LORD_OF_CHAOS")),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: AgentPrompt = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
