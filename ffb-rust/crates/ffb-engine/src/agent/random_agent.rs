//! `RandomAgent` — the parity/coverage driver. Moved out of `agent.rs` into this submodule
//! (agent module split) without behavior changes; see `agent/mod.rs` for the shared `Agent`
//! trait and module-level docs.
//!
//! `RandomAgent` mirrors the Java `ParityRunner` decision/action RNG contract (see
//! `AGENT_CONTRACT.md` and `docs/step_port/INVARIANTS.md`). A single shared instance drives BOTH
//! sides (the runner plays both coaches); its two RNG streams are kept distinct from the game
//! dice by the seed XORs below.

use std::collections::HashSet;
use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_model::prompts::AgentPrompt;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PlayerAction, SkillId};

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{canonical_setup_action, legal_block_targets, legal_foul_targets, legal_handoff_receivers, legal_pass_receivers, legal_throw_team_mate_targets, TeamSide};
use crate::step::GameState;

use super::Agent;

/// AGENT_CONTRACT §7: the pushback coach picks the min-`(x, y)` on-pitch square, deterministically
/// (no decisionRng consumed). Mirrors Java `ParityRunner.sendPushback` (keep the square with the
/// smallest x, ties broken by smallest y, over the non-locked candidates).
/// The eight neighbours of `from` that are on pitch, unoccupied by ANY player, and not already on
/// the planned path -- coordinate-sorted.
///
/// This is the same candidate rule both harnesses already apply to the first square of a move
/// (`legal_actions::adjacent_empty_move_targets` here, the neighbour loop in
/// `ParityRunner.sendMoveAction` there); it is factored out so the multimove spike walks by exactly
/// that rule at every step rather than a second, subtly different one.
///
/// Sorted by `(x, y)` and never by player id -- `AGENT_CONTRACT.md` section 6.
fn free_neighbours(
    game: &Game,
    from: FieldCoordinate,
    exclude: &[FieldCoordinate],
) -> Vec<FieldCoordinate> {
    let mut out: Vec<FieldCoordinate> = from
        .neighbours()
        .into_iter()
        .filter(|c| c.x >= 0 && c.x <= 25 && c.y >= 0 && c.y <= 14)
        .filter(|c| game.field_model.player_at(*c).is_none())
        .filter(|c| !exclude.contains(c))
        .collect();
    out.sort_by_key(|c| (c.x, c.y));
    out
}

fn choose_pushback_square(squares: &[FieldCoordinate]) -> Option<FieldCoordinate> {
    squares.iter().min_by_key(|c| (c.x, c.y)).copied()
}

/// Parity/coverage random agent. Decision RNG (`seed ^ 0xDEAD_BEEF_CAFE_0001`) drives the
/// Java-synced choices (coin guess, receive, player selection, kick target); action RNG
/// (`seed ^ 0xC0FFEE_ACE0_0001`) drives Rust-only diversity (move paths, block/foul targets).
/// Both are independent of the game-dice `GameRng`, so the agent never perturbs engine rolls.
pub struct RandomAgent {
    /// Decision RNG — synced with Java's `decisionRng`.
    decision_rng: Xoshiro256StarStar,
    /// Action-diversity RNG — independent of Java's decisions.
    action_rng: Xoshiro256StarStar,
    /// SPIKE KNOB (default 0 = off, i.e. the historical one-square behaviour).
    ///
    /// When > 1, a Move submits a path of up to this many one-step squares in a single
    /// `Action::Move`, planned ahead by the agent, instead of one square. It exists to answer ONE
    /// question cheaply -- do the two engines consume a multi-square move stack identically? --
    /// using the already-byte-matched random agent, without first porting the heuristic agent's
    /// 2,100-line scorer to Java. It drags GFI, mid-path dodges and mid-path turnovers into the
    /// parity gate for the first time.
    ///
    /// `ParityRunner.sendMoveAction` mirrors it exactly under `--multimove N`: same candidate rule,
    /// same coordinate sort, one actionRng draw per square, same MA+2 budget cap. Leaving it at 0
    /// keeps `AGENT_CONTRACT.md` untouched.
    pub multimove: usize,
    /// The squares after the first for a PRE-DRAWN (prone/rooted) multimove path. Java draws the
    /// whole path inside `sendMoveAction`, so Rust must draw the whole path at the same point in
    /// the stream -- not one square now and the rest at the Move prompt.
    pending_extra: Vec<FieldCoordinate>,
    /// Players skipped this turn because they are inactive (just recovered from STUNNED).
    /// Mirrors Java ParityRunner's `usedThisTurn` for rejected-inactive picks.
    used_this_turn: HashSet<String>,
    /// Mirrors Java ParityRunner's `justDeselected`: set when a non-Regular mini-turn
    /// (PASS_BLOCK, kickoff Blitz, …) hits its one-activation limit; the NEXT phase-1
    /// activation prompt then EndTurns immediately (consuming no picks) and clears the
    /// used-set — Java `while(true) { if (remaining.isEmpty() || justDeselected) … }`.
    just_deselected: bool,
    /// Turn key (half, turn_nr, home_playing) — detects when a new turn starts so we can
    /// clear `used_this_turn`.
    last_turn_key: Option<(i32, i32, bool)>,
    /// Java ParityRunner `eligibleThisTurn`: the player→actions eligible list is computed ONCE at
    /// the start of the turn and reused for every activation. The engine recomputes its live legal
    /// list each activation, so without this snapshot a player's cached action (e.g. BLITZ, offered
    /// when an opponent was adjacent-standing at turn start) would silently vanish after that
    /// opponent is knocked prone mid-turn — diverging from Java (seed 7 i=39: away_01 BLITZ vs Move).
    eligible_this_turn: Vec<(String, Vec<PlayerAction>)>,
    /// Debug: cumulative actionRng call count (for FFB_TRACE divergence diagnosis).
    action_rng_count: u64,
    /// Debug: cumulative decisionRng call count — compare vs Java ParityRunner.decisionRngAdvances
    /// to localize decision-stream desyncs. Counts every decision_rng draw (pick/pick_bool/KickBall).
    decision_rng_count: u64,
    /// True once the current activation is a Blitz/StandUpBlitz. Mirrors Java ParityRunner's
    /// `blitzBlockSent`: a blitzer blocks immediately and then ENDS its activation — it never
    /// spends remaining MA moving (before or after the block). Set when a Blitz action is picked,
    /// consumed by the Move-prompt handler to deselect instead of continuing to move.
    current_activation_is_blitz: bool,
    /// True once the current activation has already moved one square. Mirrors Java ParityRunner
    /// INIT_MOVING: after the first square, only the BALL CARRIER keeps moving (until MA is spent);
    /// every other player deselects. Reset at the start of every activation (ActivatePlayer).
    moved_this_activation: bool,
    /// Move target pre-drawn at activation for a PRONE (standing-up) Move — mirrors Java
    /// ParityRunner.sendMoveAction, which picks the move target at phase-2 (right after the
    /// activation) BEFORE the Select-sequence negatrait rolls (Bone Head, Really Stupid, ...). A
    /// prone player whose negatrait FAILS never reaches the Move sequence's StepInitMoving (and so
    /// never receives an AgentPrompt::Move), yet Java has already drawn that move-target actionRng.
    /// Pre-drawing here keeps the action_rng stream aligned in that case; when the Move prompt does
    /// arrive (negatrait passed / none), the square is reused without a second draw. Reset at each
    /// activation. See docs/PARITY_TTM.md "FRONTIER (human)" — the Ogre Bone-head case.
    pending_move: Option<FieldCoordinate>,
}

/// Mirror of `ParityRunner.isHandledActingAction` — the `PlayerAction`s whose
/// `sendConcreteAction` switch arm actually carries the action out. Everything else falls
/// through to its `default:` arm, which logs `UNHANDLED_ACTING_ACTION` and injects
/// `ClientCommandActingPlayer(null, null, false)` (a deselect) without touching game state.
pub(crate) fn is_handled_acting_action(pa: PlayerActionChoice) -> bool {
    matches!(
        pa,
        // MOVE / STAND_UP
        PlayerActionChoice::Move
            | PlayerActionChoice::StandUp
            // BLOCK
            | PlayerActionChoice::Block
            // BLITZ / BLITZ_MOVE / BLITZ_SELECT / STAND_UP_BLITZ
            | PlayerActionChoice::Blitz
            | PlayerActionChoice::StandUpBlitz
            // FOUL / FOUL_MOVE
            | PlayerActionChoice::Foul
            // PASS / PASS_MOVE
            | PlayerActionChoice::Pass
            // HAND_OVER / HAND_OVER_MOVE
            | PlayerActionChoice::HandOff
            // THROW_TEAM_MATE / THROW_TEAM_MATE_MOVE
            | PlayerActionChoice::ThrowTeamMate
            // KICK_TEAM_MATE / KICK_TEAM_MATE_MOVE. A kick declares through the same command and
            // the same candidate rule as a throw — every edition's `TtmMechanic.canBeKicked` is
            // `canBeThrown()` plus STANDING (plus not-rooted and own-team). Leaving it out here
            // meant both agents declared a kick and then immediately deselected it, so the mechanic
            // never executed in ANY edition and every matrix was green because of it.
            | PlayerActionChoice::KickTeamMate
            // THROW_BOMB: declares through the same ClientCommandPass as a pass; ParityRunner
            // routes it to sendPassAction.
            | PlayerActionChoice::ThrowBomb
            // HAIL_MARY_PASS: a distinct declared action for a canPassToAnySquare carrier;
            // rides the same ClientCommandPass/sendPassAction route as PASS.
            | PlayerActionChoice::HailMaryPass
            // MULTIPLE_BLOCK: two-phase declaration; targets come from the
            // MultiBlockTargets continuation prompt (2 actionRng draws).
            | PlayerActionChoice::MultipleBlock
            // TREACHEROUS / BLACK_INK (bb2020+ star specials): declared with no folded target —
            // the step finds its own victim.
            | PlayerActionChoice::RaidingParty
            | PlayerActionChoice::LookIntoMyEyes
            | PlayerActionChoice::BalefulHex
            | PlayerActionChoice::CatchOfTheDay
            | PlayerActionChoice::ThenIStartedBlastin
            | PlayerActionChoice::AllYouCanEat
            | PlayerActionChoice::Treacherous
            | PlayerActionChoice::BlackInk
            // FURIOUS_OUTPBURST (bb2025 star special): declared as a plain
            // ClientCommandActingPlayer with no folded target — StepInitFuriousOutburst shows its
            // own stab-target dialog. Omitting it here made both engines DECLARE the action and
            // then instantly deselect it (wood_elf bb2025 seed 1 i=23: Java carried it out while
            // Rust deselected and re-picked), which is the same shape that kept Kick Team-Mate
            // dead in every edition.
            | PlayerActionChoice::FuriousOutburst
            // THROW_KEG ("Beer Barrel Bash!"): declared in TWO commands, both handled by
            // StepInitSelecting — ClientCommandActingPlayer(THROW_KEG) then
            // ClientCommandThrowKeg(target), the latter publishing TARGET_PLAYER_ID. Rust folds
            // the pair into one ActivatePlayer carrying the target.
            | PlayerActionChoice::ThrowKeg
            // WISDOM_OF_THE_WHITE_DWARF: declared as ActingPlayer(MOVE) +
            // ClientCommandUseTeamMatesWisdom, mirroring BLACK_INK's command pair.
            | PlayerActionChoice::WisdomOfTheWhiteDwarf
            // AUTO_GAZE_ZOAT: declared as ActingPlayer(MOVE) + UseSkill(zoat), like BLACK_INK.
            | PlayerActionChoice::AutoGazeZoat
            // PUNT (bb2025): forceDispatch — the bare declaration pushes the punt sequence and
            // the INIT_PUNT square wait is driven by the PuntTarget contract. ParityRunner's
            // isHandledActingAction gained PUNT in the dark_elf campaign; without this mirror the
            // Rust random agent kept DESELECTING punts Java now declares (bb2025 random control
            // 94/100: all six reds `J Activate(*,PUNT)` vs a Rust deselect+re-pick).
            | PlayerActionChoice::Punt
    )
}

impl RandomAgent {
    /// Parity constructor: one shared agent for both sides, seeds matching Java byte-for-byte.
    pub fn new_parity(game_seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xDEAD_BEEF_CAFE_0001),
            action_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xC0FFEE_ACE0_0001),
            multimove: 0,
            pending_extra: Vec::new(),
            used_this_turn: HashSet::new(),
            just_deselected: false,
            last_turn_key: None,
            eligible_this_turn: Vec::new(),
            action_rng_count: 0,
            decision_rng_count: 0,
            current_activation_is_blitz: false,
            moved_this_activation: false,
            pending_move: None,
        }
    }

    /// Coverage/visual constructor (no Java sync): both streams derive deterministically from
    /// `seed`. Callers use distinct seeds per side (e.g. `seed` / `seed ^ 0xFFFF_FFFF`).
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_ACE0_0001),
            multimove: 0,
            pending_extra: Vec::new(),
            used_this_turn: HashSet::new(),
            just_deselected: false,
            last_turn_key: None,
            eligible_this_turn: Vec::new(),
            action_rng_count: 0,
            decision_rng_count: 0,
            current_activation_is_blitz: false,
            moved_this_activation: false,
            pending_move: None,
        }
    }

    /// Decision-RNG fair coin: `decisionRng.nextLong() % 2 == 0` (Java parity).
    fn pick_bool(&mut self) -> bool {
        self.decision_rng_count += 1;
        if std::env::var("FFB_TRACE").is_ok() { eprintln!("DRC_DRAW kind=bool n={}", self.decision_rng_count); }
        self.decision_rng.next_u64() % 2 == 0
    }

    /// Decision-RNG uniform index in `[0, len)`: `remainderUnsigned(nextLong(), len)`.
    fn pick(&mut self, len: usize) -> usize {
        if len == 0 { return 0; }
        self.decision_rng_count += 1;
        if std::env::var("FFB_TRACE").is_ok() { eprintln!("DRC_DRAW kind=pick len={} n={}", len, self.decision_rng_count); }
        (self.decision_rng.next_u64() as usize) % len
    }

    /// Action-RNG uniform index — diversity picks (move target, block/foul target).
    /// Plan up to `multimove` squares in ONE `Action::Move`, drawing one `action_rng` pick per
    /// extra square. Byte-mirrored by `ParityRunner.sendMoveAction` under `--multimove N`.
    ///
    /// The candidate rule at every step is the same one both harnesses already use for the FIRST
    /// square -- the eight neighbours of the square being planned from, on pitch, unoccupied by any
    /// player, coordinate-sorted -- plus "not already somewhere on this path", because the agent is
    /// planning ahead against a board that has not moved yet.
    ///
    /// Capped at the player's MA + 2 (the two rushes), so the agent never proposes a path the
    /// engine must refuse. `path` already contains the first square when this is called.
    fn extend_multimove(
        &mut self,
        game: &Game,
        player_id: &str,
        path: &mut Vec<FieldCoordinate>,
        spent: i32,
    ) {
        let ma = match game.player(player_id) {
            Some(p) => p.movement_with_modifiers(),
            None => return,
        };
        let budget = (ma + 2 - spent).max(0) as usize;
        let want = self.multimove.min(budget);
        while path.len() < want {
            let from = *path.last().expect("path is never empty here");
            let cands = free_neighbours(game, from, path);
            if cands.is_empty() {
                break;
            }
            let i = self.pick_action(cands.len());
            path.push(cands[i]);
        }
    }

    /// The parity contract's Throw/Kick-Team-Mate thrown-player pick: adjacent standing
    /// throwable teammates (`legal_throw_team_mate_targets`, coordinate-sorted), ONE actionRng
    /// draw — 1:1 with `ParityRunner.sendThrowTeamMateAction`. `pub(crate)` because the HEURISTIC
    /// folds the same pick into its declaration: Java's heuristic driver reaches the identical
    /// code (sendConcreteAction THROW_TEAM_MATE → sendThrowTeamMateAction, one actionRng) at
    /// phase 2, while Rust's default-arm candidate carries no target — without this fold the
    /// declaration deselected instantly and no TTM ever resolved under the heuristic
    /// (chaos_pact: Java threw goblins, Rust's big guys did nothing — bb2020 seeds 6/7/10/19,
    /// bb2016 seed 4).
    pub(crate) fn fold_ttm_target(&mut self, game: &ffb_model::model::game::Game, player_id: &str) -> Option<String> {
        let side = if game.home_playing { TeamSide::Home } else { TeamSide::Away };
        let targets = legal_throw_team_mate_targets(game, player_id, side);
        if targets.is_empty() {
            None
        } else {
            let tidx = self.pick_action(targets.len());
            Some(targets[tidx].clone())
        }
    }

    /// The parity contract's Beer Barrel Bash target pick: opponents within 3, STANDING
    /// (`legal_throw_keg_targets`, coordinate-sorted), ONE actionRng draw — 1:1 with
    /// `ParityRunner.sendStarSpecialDeclaration`'s THROW_KEG arm (kegIdx =
    /// remainderUnsigned(actionRng.nextLong(), kegTargets.size())). `pub(crate)` because the
    /// HEURISTIC folds the same pick into its declaration: Java's heuristic driver reaches the
    /// identical arm via sendStarSpecialDeclaration, while Rust's candidate carries no target —
    /// without this fold the keg ran with a NULL target and spent no target draw (dwarf bb2025
    /// seed 2: kegThrow target_id=null, streams 4 draws apart at the keg re-roll offer).
    /// Empty targets → None (Java deselects with NO kegIdx draw).
    pub(crate) fn fold_keg_target(&mut self, game: &ffb_model::model::game::Game, player_id: &str) -> Option<String> {
        let targets = crate::legal_actions::legal_throw_keg_targets(game, player_id);
        if targets.is_empty() {
            None
        } else {
            let tidx = self.pick_action(targets.len());
            Some(targets[tidx].clone())
        }
    }

    /// The parity contract's pass-target pick — `ParityRunner.sendPassAction`: on-pitch
    /// teammates, coordinate-sorted, ONE actionRng draw. PASS / THROW_BOMB / HAIL_MARY_PASS /
    /// ALL_YOU_CAN_EAT all ride it (phase 2 routes them to the same method). `pub(crate)`
    /// because the HEURISTIC's immediate declarations (no move variant: HMP, bomb) reach the
    /// identical Java code while Rust's candidate carries no target — dwarf bb2016 seed 20:
    /// Java's Farblast HMP targeted (12,9) via this rule, Rust's picked its own square and the
    /// scatter walk diverged. Empty → None.
    pub(crate) fn fold_pass_receiver(&mut self, game: &ffb_model::model::game::Game, player_id: &str) -> Option<String> {
        let side = if game.home_playing { TeamSide::Home } else { TeamSide::Away };
        let receivers = legal_pass_receivers(game, player_id, side);
        if receivers.is_empty() {
            None
        } else {
            let ridx = self.pick_action(receivers.len());
            Some(receivers[ridx].clone())
        }
    }

    fn pick_action(&mut self, len: usize) -> usize {
        self.action_rng_count += 1;
        if len == 0 { 0 } else { (self.action_rng.next_u64() as usize) % len }
    }

    /// T2 parity: consume exactly 1 decisionRng draw (player pick), no actionRng.
    /// Mirrors Java T2's one-player-pick-then-deselect-then-EndTurn pattern so the
    /// decisionRng stream stays synced for the half-2 kickoff.
    pub fn pick_t2_activation(&mut self, n: usize) {
        let _ = self.pick(n);
    }
}

impl Agent for RandomAgent {
    fn act(&mut self, gs: &GameState) -> Action {
        match gs.current_prompt() {
            // Pregame decisions both draw the decision RNG once (AGENT_CONTRACT.md §2).
            Some(AgentPrompt::CoinChoice { .. }) => Action::CoinChoice { heads: self.pick_bool() },
            Some(AgentPrompt::ReceiveChoice { .. }) => Action::ReceiveChoice { receive: self.pick_bool() },
            // Java parity: the kicking coach picks a target in the opponent's half — two
            // decisionRng draws (x then y), x offset into the receiving half. 1:1 with the
            // monolith's KickBall handler so the decisionRng stream stays synced with Java.
            Some(AgentPrompt::KickBall) => {
                self.decision_rng_count += 2;
                let x_raw = (self.decision_rng.next_u64() % 13) as i32;
                let y_raw = (self.decision_rng.next_u64() % 13) as i32;
                let x = if gs.game.home_playing { x_raw + 13 } else { x_raw };
                // Java ParityRunner: `home ? kickCoord : kickCoord.transform()`. The kick target is
                // built in server frame (away kicks into home's half, x 0..12); an away "client" sends
                // it in client frame, so StepKickoff's own transform (applied when !home_playing) maps
                // it back. Without this pre-transform, StepKickoff mirrored (6,8)->(19,8), landing the
                // ball in the kicking half -> spurious touchback and a diverged half-2 kickoff.
                let coord = FieldCoordinate::new(x, y_raw + 1);
                let coord = if gs.game.home_playing { coord } else { coord.transform() };
                Action::KickBall { coord }
            }
            // AGENT_CONTRACT.md §4-5: 1 decisionRng for player pick over remaining (§4 — EndTurn
            // is automatic when remaining is empty, NOT an explicit pick option),
            // 1 actionRng for action pick, 1 actionRng for block target when Block/Blitz.
            //
            // Java inactive-skip (ParityRunner tier>=3): players that are PRONE with active=false
            // (just recovered from STUNNED this turn) are in the eligible list but rejected when
            // picked. Each rejection consumes 1 decisionRng call. `used_this_turn` tracks
            // rejected players across multiple InitSelecting calls within the same turn.
            Some(AgentPrompt::ActivatePlayer { eligible_players }) => {
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_START arc={}", self.action_rng_count);
                }
                // Detect new turn and clear the skip-set.
                let turn_nr = if gs.game.home_playing {
                    gs.game.turn_data_home.turn_nr
                } else {
                    gs.game.turn_data_away.turn_nr
                };
                // Java ParityRunner: `if (turn < 1) { inject EndTurn; break; }` — BEFORE the
                // turn-key update. Fires when a pass-block window opens for a team that has
                // not started a turn this half yet (counter still 0): the window closes with
                // zero movers and zero rng draws (amazon seed 1 i=173, home10's pass while
                // away's half-2 counter was 0).
                if turn_nr < 1 {
                    return Action::EndTurn;
                }
                let turn_key = (gs.game.half, turn_nr, gs.game.home_playing);
                if self.last_turn_key != Some(turn_key) {
                    self.last_turn_key = Some(turn_key);
                    self.used_this_turn.clear();
                    // Java: eligibleThisTurn = computeEligiblePlayers(game) — snapshot once at turn start.
                    self.eligible_this_turn = eligible_players.clone();
                }
                // Java ParityRunner: non-Regular modes (PASS_BLOCK, kickoff Blitz!, QuickSnap)
                // allow ONE activation, then EndTurn with justDeselected set.
                if gs.game.turn_mode != ffb_model::enums::TurnMode::Regular
                    && !self.used_this_turn.is_empty()
                {
                    self.just_deselected = true;
                    return Action::EndTurn;
                }
                // Pick from the turn-start snapshot (Java `eligibleThisTurn`), NOT the engine's live
                // per-activation list, so an action offered at turn start (e.g. BLITZ) survives even if
                // its target is knocked down later in the same turn. Clone to sidestep the &mut self pick.
                let eligible_players = self.eligible_this_turn.clone();

                // Build `remaining` as indices into the snapshot, excluding already-skipped.
                // Wrapped in 'reselect so a committed-but-untargetable action (e.g. a FOUL whose
                // victim moved away since the turn-start snapshot) can DESELECT and pick another
                // player without ending the turn — mirroring ParityRunner's deselect commands.
                'reselect: loop {
                let mut remaining: Vec<usize> = (0..eligible_players.len())
                    .filter(|&i| !self.used_this_turn.contains(&eligible_players[i].0))
                    .collect();

                // Inactive-skip loop (mirrors Java ParityRunner while(true) pick loop).
                let (player_id, actions) = loop {
                    // Java: `if (remaining.isEmpty() || justDeselected) { justDeselected = false;
                    // usedThisTurn.clear(); inject EndTurn; }` — the justDeselected follow-on
                    // also ends the ORIGINAL team's turn right after a pass-block window closes
                    // (the pass was that turn's last processed activation in Java).
                    if remaining.is_empty() || self.just_deselected {
                        self.just_deselected = false;
                        self.used_this_turn.clear();
                        return Action::EndTurn;
                    }
                    let pick = self.pick(remaining.len()); // consumes 1 decisionRng
                    let player_list_idx = remaining.remove(pick);
                    let (pid, acts) = &eligible_players[player_list_idx];
                    // Java: usedThisTurn.add(playerId) for EVERY pick — successful activations
                    // included — so the remaining list shrinks each activation and EndTurn fires
                    // once every player has been picked once. Without this the agent re-activates
                    // the same players forever and the turn never ends.
                    self.used_this_turn.insert(pid.clone());
                    // Skip a picked player whose PlayerState is inactive — mirrors Java
                    // ParityRunner tier>=3 `!pickedState.isActive()` (SKIP_INACTIVE), which does
                    // NOT restrict to prone. A player just recovered from STUNNED (STUNNED→PRONE)
                    // is prone+inactive, but a team-mate THROWN this turn lands STANDING yet
                    // active=false (it already used its activation via the throw) — Java rejects
                    // that pick, so Rust must too (ogre seed 1 i=11: away_09, thrown by away_05's
                    // TTM, must not re-activate). Restricting to prone let the thrown player act again.
                    let ps = gs.game.field_model.player_state(pid);
                    let is_inactive = ps.map(|s| !s.is_active()).unwrap_or(false);
                    if std::env::var("FFB_ACT_TRACE").is_ok() {
                        eprintln!("RUST_ACT_PICK pid={} pick={} N={} inactive={} acts={:?}",
                            pid, pick, remaining.len()+1, is_inactive, acts);
                    }
                    if is_inactive {
                        // Rejected: decisionRng already consumed; excluded for the rest of the turn.
                        continue;
                    }
                    break (pid, acts);
                };
                // Filter stale actions: mirrors Java ParityRunner.filterStaleActions — removes
                // Blitz/Block if blitz_used, Pass if pass_used, etc. The eligible
                // list was captured at turn start, so single-use actions may already be consumed.
                let td = if gs.game.home_playing { &gs.game.turn_data_home } else { &gs.game.turn_data_away };
                let live_actions: Vec<PlayerAction> = actions.iter().filter(|a| match a {
                    PlayerAction::Block | PlayerAction::Blitz | PlayerAction::StandUpBlitz => !td.blitz_used,
                    PlayerAction::Pass | PlayerAction::HailMaryPass => !td.pass_used,
                    PlayerAction::HandOver => !td.hand_over_used,
                    PlayerAction::Foul => !td.foul_used,
                    // Throw/Kick Team-Mate are once per team turn (set ttm_used/ktm_used). The
                    // turn-start snapshot can still offer a second one after the first Ogre throws;
                    // filter it out as stale, or the engine rejects the second throw and the harness
                    // loops (ogre seed 1: away_06's throw after away_05 already threw).
                    // BB2016 spends the team's PASS on a Throw Team-Mate (bb2016
                    // ThrowTeamMateBehaviour sets passUsed), and bb2016 StepInitSelecting rejects the
                    // command once the pass is gone. Mirrors ParityRunner.filterStaleActions.
                    // BB2020 spends the pass exactly as BB2016 does (bb2020
                    // ThrowTeamMateBehaviour sets passUsed; bb2020 TtmMechanic.isTtmAvailable is
                    // `!turnData.isPassUsed()`); only BB2025 tracks TTM on its own flag.
                    PlayerAction::ThrowTeamMate => {
                        if gs.game.rules == ffb_model::enums::Rules::Bb2025 {
                            !td.ttm_used
                        } else {
                            !td.ttm_used && !td.pass_used
                        }
                    }
                    // BB2016 spends the team's BLITZ on a Kick Team-Mate
                    // (`bb2016/TtmMechanic.isKtmAvailable` is `!turnData.isBlitzUsed()`); BB2020 and
                    // BB2025 track it on their own flag.
                    PlayerAction::KickTeamMate => {
                        if gs.game.rules == ffb_model::enums::Rules::Bb2016 {
                            !td.blitz_used
                        } else {
                            !td.ktm_used
                        }
                    }
                    _ => true,
                }).cloned().collect();
                // Non-REGULAR window modes (PASS_BLOCK): the harness contract shrinks the
                // action list to MOVE + the UseSkill specials. A window Block/Blitz/Foul was
                // always a declare-then-deselect no-op, but its declaration is a live grenade:
                // a window BLITZ against the SUSPENDED THROWER re-fires Java's
                // CONFIRM_END_ACTION dialog forever (dark_elf bb2020 seed 61 i=217 — the Java
                // harness hit its 2M-iteration cap). ParityRunner filters its snapshot with the
                // identical rule so idx % N stays aligned.
                let live_actions: Vec<PlayerAction> = if gs.game.turn_mode
                    == ffb_model::enums::TurnMode::PassBlock
                {
                    live_actions.into_iter().filter(|a| matches!(a,
                        PlayerAction::Move | PlayerAction::Treacherous | PlayerAction::BlackInk
                    )).collect()
                } else {
                    live_actions
                };
                let action_idx = self.pick_action(live_actions.len());
                let player_action = player_action_to_pac(&live_actions[action_idx]);
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_PICK pid={player_id} N={} idx={action_idx} action={player_action:?} arc={} drc={}", live_actions.len(), self.action_rng_count, self.decision_rng_count);
                }
                // For Block/Blitz: pick target from adjacent opponents
                // For Foul: pick foul target from adjacent prone/stunned opponents (1 actionRng call)
                let block_defender_id = match player_action {
                    // BLITZ no longer folds its target. Java routes an untargeted BLITZ_MOVE
                    // through BLITZ_SELECT, and StepSelectBlitzTarget asks for the target with the
                    // SAME single actionRng draw at the SAME stream position (before the negatrait
                    // rolls). Folding it here as well would spend the draw twice. BACKLOG §12.
                    // BB2016 has NO SelectBlitzTarget step at all - ParityRunner drives it as a
                    // 3-command blitz (CLIENT_BLITZ_MOVE then CLIENT_BLOCK) and BB2016 has its own
                    // StepInitSelecting, so no chain prompt ever arrives to supply the target.
                    // Leaving it unfolded there strands every BB2016 blitz with no defender:
                    // measured 0/30 rosters, diverging at STEP 1 on nearly all of them, against a
                    // green main. The agent is shared by all three editions, so the change has to
                    // be gated rather than global.
                    PlayerActionChoice::Blitz if gs.game.rules != ffb_model::enums::Rules::Bb2016 => None,
                    PlayerActionChoice::Blitz
                    | PlayerActionChoice::Block
                    | PlayerActionChoice::StandUpBlitz => {
                        let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let targets = legal_block_targets(&gs.game, player_id, side);
                        if targets.is_empty() {
                            None
                        } else {
                            let tidx = self.pick_action(targets.len());
                            if std::env::var("FFB_TRACE").is_ok() {
                                let attacker_coord = gs.game.field_model.player_coordinate(player_id).map(|c| format!("({},{})", c.x, c.y)).unwrap_or_default();
                                let all_targets: Vec<String> = targets.iter().map(|t| {
                                    let tc = gs.game.field_model.player_coordinate(t).map(|c| format!("({},{})", c.x, c.y)).unwrap_or_default();
                                    format!("{}@{}", t, tc)
                                }).collect();
                                eprintln!("RUST_BLOCK_PICK pid={} attacker={} N={} idx={} def={} all=[{}] arc={}", player_id, attacker_coord, targets.len(), tidx, targets[tidx], all_targets.join(","), self.action_rng_count);
                            }
                            Some(targets[tidx].clone())
                        }
                    }
                    PlayerActionChoice::Foul => {
                        let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let targets = legal_foul_targets(&gs.game, player_id, side);
                        if targets.is_empty() {
                            None
                        } else {
                            let tidx = self.pick_action(targets.len());
                            Some(targets[tidx].clone())
                        }
                    }
                    PlayerActionChoice::HandOff => {
                        let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let receivers = legal_handoff_receivers(&gs.game, player_id, side);
                        if receivers.is_empty() {
                            None
                        } else {
                            let ridx = self.pick_action(receivers.len());
                            Some(receivers[ridx].clone())
                        }
                    }
                    // ThrowBomb rides the Pass arm: ParityRunner's sendConcreteAction routes
                    // THROW_BOMB to the very same sendPassAction, so the candidate rule and the
                    // single actionRng draw must match a pass exactly. Declaring a bomb with no
                    // target left StepInitPassing parked with an unset thrower and NO prompt, and
                    // the parity loop breaks silently on a prompt-less park -- the game simply
                    // ended at the first bomb.
                    // HAIL_MARY_PASS rides it too: ParityRunner routes the declaration to the
                    // same sendPassAction (same candidates, same single actionRng draw).
                    // ALL_YOU_CAN_EAT delegates to THROW_BOMB and declares through the same
                    // route — Java's phase-2 keys on the ACTING action (already the delegate),
                    // so its sendPassAction fires identically; the folded target must too.
                    PlayerActionChoice::Pass | PlayerActionChoice::ThrowBomb
                    | PlayerActionChoice::AllYouCanEat
                    | PlayerActionChoice::HailMaryPass => {
                        let r = self.fold_pass_receiver(&gs.game, player_id);
                        if std::env::var("FFB_ACT_TRACE").is_ok() {
                            eprintln!("RPASSPICK pid={} recv={:?}", player_id, r);
                        }
                        r
                    }
                    // Throw/Kick Team-Mate: pick the thrown player (an adjacent standing Right Stuff
                    // teammate), coordinate-sorted, 1 actionRng. Empty → None → StepInitSelecting
                    // deselects (no valid throwable teammate). The target square is chosen later, on
                    // the ThrowTeamMateTarget prompt. 1:1 with ParityRunner.sendThrowTeamMateAction.
                    // A KICK uses the same candidate rule as a throw: every edition's
                    // `TtmMechanic.canBeKicked` is `canBeThrown()` plus STANDING (plus not-rooted and
                    // own-team), which is what `legal_throw_team_mate_targets` already computes, and
                    // what ParityRunner.sendThrowTeamMateAction sends for both. Picking a target for
                    // the throw only meant a declared KICK carried no thrown player at all, so the
                    // sequence stalled and the kick silently never resolved.
                    // Beer Barrel Bash!: the client declares in two steps — ActingPlayer(THROW_KEG)
                    // puts the client in its THROW_KEG state, then the coach clicks a target and
                    // ClientCommandThrowKeg carries it. Both land in StepInitSelecting, so the
                    // folded target here IS that second command. Candidates are
                    // ThrowKegLogicModule.isValidTarget (<=3 steps, STANDING, opposing team),
                    // coordinate-sorted, 1 actionRng — identical to ParityRunner's arm.
                    // Empty → None → deselect below (Java has no valid click either).
                    PlayerActionChoice::ThrowKeg => self.fold_keg_target(&gs.game, player_id),
                    PlayerActionChoice::ThrowTeamMate | PlayerActionChoice::KickTeamMate => {
                        self.fold_ttm_target(&gs.game, player_id)
                    }
                    _ => None,
                };
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_END arc={}", self.action_rng_count);
                }
                // Mirror ParityRunner.sendConcreteAction's `default:` arm. Its switch handles only
                //   MOVE, STAND_UP, BLOCK, BLITZ, BLITZ_MOVE, BLITZ_SELECT, STAND_UP_BLITZ,
                //   FOUL(_MOVE), PASS(_MOVE), HAND_OVER(_MOVE), THROW_TEAM_MATE(_MOVE)
                // and every other PlayerAction falls through to
                //   `UNHANDLED_ACTING_ACTION: <pa> — deselecting`
                //   MatchRunner.inject(new ClientCommandActingPlayer(null, null, false));
                // i.e. the exact same deselect the no-target FOUL below uses: the player-pick
                // decisionRng and action-pick actionRng are already spent, the player is already in
                // `used_this_turn`, and the team's turn keeps going with a fresh pick.
                //
                // Rust instead CARRIED OUT the action. For the Goblin Bombardier's THROW_BOMB that
                // meant Java burned a no-op step and re-activated while Rust threw the bomb — and
                // then ended the game outright (goblin bb2016 seed 1: `game_end` at i=3 where Java
                // plays on to i=901). Deselecting mirrors the harness exactly.
                if !is_handled_acting_action(player_action) {
                    if std::env::var("FFB_TRACE").is_ok() {
                        eprintln!("RUST_UNHANDLED_ACTION_DESELECT pid={player_id} action={player_action:?}");
                    }
                    continue 'reselect;
                }
                // Mirror ParityRunner.sendFoulAction: the turn-start eligible snapshot may still
                // offer FOUL for a player whose only adjacent prone/stunned victim has since moved
                // or stood up. When that leaves NO legal foul target, Java does NOT commit the
                // foul — it injects ClientCommandActingPlayer(null,null,false), a DESELECT that
                // leaves the team's turn going (unlike a no-target BLITZ, handled in StepInitSelecting,
                // which ends the turn). The player-pick decisionRng and the action-pick actionRng are
                // already consumed (Java picks player+action before sendFoulAction deselects), and the
                // player is already in `used_this_turn`, so re-picking chooses a different player.
                // A THROW_KEG with no valid target: the client would have no square to click, so
                // the coach never sends ClientCommandThrowKeg and the declaration never completes.
                // ParityRunner deselects in the same situation; without this the sequence would
                // run with a null TARGET_PLAYER_ID.
                if matches!(player_action, PlayerActionChoice::ThrowKeg) && block_defender_id.is_none() {
                    if std::env::var("FFB_TRACE").is_ok() {
                        eprintln!("RUST_KEG_DESELECT pid={player_id} (no valid keg target)");
                    }
                    continue 'reselect;
                }
                if matches!(player_action, PlayerActionChoice::Foul) && block_defender_id.is_none() {
                    if std::env::var("FFB_TRACE").is_ok() {
                        eprintln!("RUST_FOUL_DESELECT pid={player_id} (no legal foul target)");
                    }
                    continue 'reselect;
                }

                // Same staleness as the no-target FOUL above, and it is not merely a parity
                // issue: the turn-start eligible snapshot can still offer FURIOUS_OUTPBURST after
                // the team's blitz has been spent, or after the star's targets have moved out of
                // range. Declaring it then makes STOCK JAVA CRASH — every abort path in the
                // bb2025 FuriousOutburst sequence jumps to the `END` label, which IS
                // StepEndFuriousOutburst, and that step dereferences
                // `fieldModel.getTargetSelectionState().getSelectedPlayerId()` unconditionally
                // (NullPointerException, ffb-server StepEndFuriousOutburst:71 — it killed the
                // batched JVM mid-run at wood_elf bb2025 seed 65 i=190, `f1000,0000`).
                // The real client never reaches it because SelectLogicModule re-evaluates
                // isFuriousOutburstAvailable at click time; ParityRunner's sendConcreteAction now
                // does the same and deselects, so mirror it here.
                if matches!(player_action, PlayerActionChoice::FuriousOutburst)
                    && !crate::legal_actions::is_furious_outburst_available(&gs.game, &player_id)
                {
                    if std::env::var("FFB_TRACE").is_ok() {
                        eprintln!("RUST_FO_DESELECT pid={player_id} (stale turn-start offer)");
                    }
                    continue 'reselect;
                }
                // Java ParityRunner: a Blitz blocks immediately then ends the activation (no MA
                // spent moving). Remember this so the follow-up Move prompt deselects instead of
                // wandering with remaining movement.
                self.current_activation_is_blitz = matches!(
                    player_action,
                    PlayerActionChoice::Blitz | PlayerActionChoice::StandUpBlitz
                );
                // New activation → reset the "moved one square" tracker (Java INIT_MOVING policy).
                self.moved_this_activation = false;
                self.pending_move = None;
                self.pending_extra.clear();
                // Prone (standing-up) Move: mirror Java ParityRunner.sendMoveAction, which draws the
                // move target at phase-2 — BEFORE the Select-sequence negatrait rolls. The Rust engine
                // emits AgentPrompt::Move only from the Move sequence's StepInitMoving, which a prone
                // player never reaches when its negatrait (Bone Head, Really Stupid, ...) FAILS in the
                // Select sequence. Pre-drawing the move-target actionRng here keeps the stream aligned
                // in that case; the AgentPrompt::Move handler reuses this square with no second draw.
                // (Standing players already draw at StepInitMoving BEFORE their Move-sequence negatrait,
                // so they need no pre-draw.) legal_move_targets is coordinate-based, so the pre-activation
                // list equals the post-stand-up list the engine would offer — same list, same pick.
                // A PRONE or ROOTED player's Move activation never reaches StepInitMoving's Move prompt
                // in Rust (a prone player's negatrait can fail in the Select sequence; a rooted player
                // cannot leave its square), yet Java's ParityRunner.sendMoveAction ALWAYS pre-draws the
                // move target at phase-2 when it commits to a MOVE activation. Mirror that draw here so
                // the action-RNG stream stays aligned. PRONE: store the target — StepInitMoving DOES
                // prompt after the stand-up, and reuses it (no 2nd draw). ROOTED: the target is drawn
                // and DISCARDED — there is no follow-up Move prompt to consume it (wood_elf seed 1 i=207:
                // rooted away_01 Treeman; Java drew a target, Rust drew nothing → arc desync shifted the
                // next mover's square pick). Standing (non-rooted) players draw at StepInitMoving as usual.
                if matches!(player_action, PlayerActionChoice::Move) {
                    let st = gs.game.field_model.player_state(player_id);
                    let is_prone = st.map(|s| s.is_prone()).unwrap_or(false);
                    let is_rooted = st.map(|s| s.is_rooted()).unwrap_or(false);
                    if is_prone || is_rooted {
                        // NO-CAP neighbour list: at pre-draw time acting_player is still the PREVIOUS
                        // activator, so legal_move_targets's MA cap reads a stale current_move and can
                        // wrongly zero the list. A fresh activation hasn't spent MA — mirror ParityRunner.
                        let targets = crate::legal_actions::adjacent_empty_move_targets(&gs.game, player_id);
                        // Java sendMoveAction deselects with 0 actionRng when there is no adjacent empty
                        // square; only draw when the candidate list is non-empty.
                        if !targets.is_empty() {
                            let idx = self.pick_action(targets.len());
                            // Store the pre-drawn square for BOTH the prone and the rooted case.
                            // ParityRunner.sendMoveAction draws exactly ONE actionRng target per
                            // activation; the Move-prompt handler reuses `pending_move` with no second
                            // draw. Storing it only for prone players meant a ROOTED player's pre-draw
                            // was discarded and the Move prompt drew AGAIN — 3 actionRng calls where
                            // Java makes 2, permanently shifting the agent stream (wood_elf bb2016
                            // seed 2 step 64: the rooted Treeman home_01 pre-drew (13,6) — the square
                            // Java uses — then re-drew (11,7); from there every later target pick was
                            // off, first visible as home_10 moving to (3,7) instead of (5,7) at i=65).
                            self.pending_move = Some(targets[idx]);
                            // SPIKE: Java's sendMoveAction draws the WHOLE path here, so the extra
                            // squares must be drawn at this point in the stream too, not later at
                            // the Move prompt. `spent` is 0, not `acting_player.current_move`: at
                            // pre-draw time the acting player is still the PREVIOUS activator (the
                            // same staleness the no-cap candidate list above works around), while
                            // Java reads the fresh phase-2 value of a just-activated player.
                            if self.multimove > 1 {
                                // Java's `sendMoveAction` reads `ap.getCurrentMove()` at the moment
                                // it draws, and for a PRONE player that moment is AFTER the stand-up:
                                // measured `JAVA_PATH len=5 currentMove=3` against Rust's `len=6`
                                // (lineman bb2025 seed 1, --multimove 6). Rust pre-draws BEFORE the
                                // stand-up, so it has to charge the same 3 movement Java has already
                                // spent, or it plans one square too many and rushes a third time.
                                //
                                // A ROOTED player never moves at all, so nothing is spent there.
                                let spent = if is_prone { crate::util::movement_calc::MovementCalc::STAND_UP_COST } else { 0 };
                                let mut path = vec![targets[idx]];
                                self.extend_multimove(&gs.game, player_id, &mut path, spent);
                                self.pending_extra = path[1..].to_vec();
                            }
                            if std::env::var("FFB_TRACE").is_ok() {
                                let tag = if is_prone { "prone_predraw" } else { "rooted_predraw" };
                                eprintln!("RUST_SMA pid={} N={} {}", player_id, targets.len(), tag);
                                eprintln!("RUST_PICK pid={} N={} idx={} t=({},{}) {}", player_id, targets.len(), idx, targets[idx].x, targets[idx].y, tag);
                            }
                        }
                    }
                }
                break 'reselect Action::ActivatePlayer { player_id: player_id.clone(), player_action, block_defender_id };
                }
            }
            // Move prompt: pick destination from legal squares using actionRng.
            // 1:1 mirror of the reference harness ParityRunner.sendMoveAction + its INIT_MOVING
            // handler: pick ONE square uniformly from all unoccupied on-pitch neighbours (already
            // sorted by (x,y)), then ALWAYS deselect on the follow-up prompt. There is no
            // carrier-advance bias and no multi-square carrier-continue — those were mirrored
            // against an older modified ParityRunner and diverge from the stock harness (seed 7
            // i=11: away_03 dodge picked idx=0 of a 2-square advancing subset instead of idx=2
            // of the full 6-square list). See ParityRunner.sendMoveAction (no bias) + the
            // INIT_MOVING case (always injects ClientCommandActingPlayer(null,null,false)).
            Some(AgentPrompt::Move { player_id, squares }) => {
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_SMA pid={} N={}", player_id, squares.len());
                }
                // Pass-block window (On The Ball): Java's engine flow never re-presents
                // INIT_SELECTING phase 2 for the window mover, so ParityRunner's INIT_MOVING
                // handler deselects immediately — the mover activates but never moves and no
                // target is drawn (amazon seeds 8/11: the OTB defender stays put in Java).
                if gs.game.turn_mode == ffb_model::enums::TurnMode::PassBlock {
                    return Action::EndPlayerAction;
                }
                // Blitz: with the two-phase declaration the target was chosen at
                // SELECT_BLITZ_TARGET (which spent the actionRng pick) and the acting action is
                // now BLITZ_MOVE, so this Move prompt is where the BLOCK is issued — exactly what
                // ParityRunner does: "the target was already chosen at SELECT_BLITZ_TARGET;
                // CLIENT_BLOCK with a targetSelectionState dispatches as BLITZ". Deselecting here
                // (the old folded-declaration behaviour) ended the blitz having done nothing.
                if self.current_activation_is_blitz {
                    if let Some(def) = gs.game.field_model.target_selection_state.as_ref()
                        .and_then(|ts| ts.get_selected_player_id().cloned())
                    {
                        self.current_activation_is_blitz = false;
                        self.moved_this_activation = true;
                        return Action::Block { defender_id: def };
                    }
                    self.current_activation_is_blitz = false;
                    self.moved_this_activation = true;
                }
                // Prone (standing-up) Move: the target was pre-drawn at activation (mirroring Java's
                // phase-2 sendMoveAction). Reuse it here — no second actionRng draw — then the
                // follow-up prompt deselects via moved_this_activation, one move per activation.
                if let Some(sq) = self.pending_move.take() {
                    if self.moved_this_activation {
                        return Action::EndPlayerAction;
                    }
                    self.moved_this_activation = true;
                    if std::env::var("FFB_TRACE").is_ok() {
                        eprintln!("RUST_MOVE_PRE pid={} t=({},{})", player_id, sq.x, sq.y);
                    }
                    let mut path = vec![sq];
                    path.extend(self.pending_extra.drain(..));
                    if std::env::var_os("FFB_TRACE").is_some() {
                        eprintln!("RUST_PATH pid={} len={} (predrawn)", player_id, path.len());
                    }
                    return Action::Move { path };
                }
                if squares.is_empty() {
                    // No adjacent empty square: deselect, ending the activation — 1:1 with Java
                    // ParityRunner.sendMoveAction's `ClientCommandActingPlayer(null, null, false)`.
                    // (An empty Move { path: [] } is a no-op the step ignores and re-prompts,
                    // looping forever.) 0 RNG consumed on either side.
                    return Action::EndPlayerAction;
                }
                // Java ParityRunner INIT_MOVING always deselects after the first square — one move
                // per activation, then the activation ends. No carrier keeps moving.
                if self.moved_this_activation {
                    return Action::EndPlayerAction;
                }
                // ParityRunner.sendMoveAction picks uniformly from ALL unoccupied on-pitch
                // neighbours (sorted by (x,y)); no carrier-advance filtering.
                let idx = self.pick_action(squares.len());
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_PICK pid={} N={} idx={} t=({},{})", player_id, squares.len(), idx, squares[idx].x, squares[idx].y);
                }
                self.moved_this_activation = true;
                let mut path = vec![squares[idx]];
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_PATHPRE pid={} currentMove={}", player_id, gs.game.acting_player.current_move);
                }
                // SPIKE (see `multimove`): keep walking, one actionRng draw per extra square,
                // mirrored by ParityRunner.sendMoveAction under `--multimove N`.
                if self.multimove > 1 {
                    self.extend_multimove(
                        &gs.game, &player_id, &mut path, gs.game.acting_player.current_move);
                }
                if std::env::var_os("FFB_TRACE").is_some() {
                    eprintln!("RUST_PATH pid={} len={}", player_id, path.len());
                }
                Action::Move { path }
            }
            // Pushback: AGENT_CONTRACT §7 — choose the min-(x,y) on-pitch square, DETERMINISTICALLY.
            // Java ParityRunner.sendPushback iterates the non-locked pushback squares and keeps the one
            // with the smallest x (ties broken by smallest y) and consumes ZERO decisionRng calls. The
            // previous code randomly indexed a sorted list AND consumed a decision_rng call, which both
            // picked the wrong square and desynced the decision_rng stream for every later pick.
            Some(AgentPrompt::Pushback { squares, .. }) => {
                match choose_pushback_square(squares) {
                    Some(coord) => Action::PushTo { coord },
                    None => Action::Acknowledge,
                }
            }
            // Follow-up: AGENT_CONTRACT §7 — ALWAYS DECLINE, deterministically, consuming 0 rng.
            // Java ParityRunner FOLLOWUP_CHOICE = `sendFollowupChoice(false)` (no decisionRng draw).
            // The old code random-sampled (pick_bool) AND consumed a decision_rng call, which both
            // sometimes followed up into the pushed player's vacated square (wrong final position)
            // and desynced the decision stream.
            // Throw Team-Mate target: the thrower has picked up the teammate; choose where to throw.
            // Deterministic — 3 squares toward the opponent end zone (quick-pass range, always legal),
            // clamped to the pitch, 0 actionRng. Sent in the acting client's view (canonical for home,
            // mirrored for away) so StepInitThrowTeamMate's un-mirror yields the canonical target,
            // matching ParityRunner.sendMoveAction's coordinate convention.
            Some(AgentPrompt::ThrowTeamMateTarget { thrower_id, thrown_player_id }) => {
                let is_home = gs.game.team_home.player(thrower_id).is_some();
                let dir = if is_home { 1 } else { -1 };
                let target = gs.game.field_model.player_coordinate(thrower_id)
                    .map(|c| FieldCoordinate::new((c.x + dir * 3).clamp(0, 25), c.y.clamp(0, 14)))
                    .unwrap_or(FieldCoordinate::new(0, 0));
                let cmd_coord = if is_home { target } else { target.transform() };
                Action::ThrowTeamMate { player_id: thrown_player_id.clone(), coord: cmd_coord }
            }
            // Swoop target (BB2016/BB2020): Java's mixed StepSwoop offers the (at most four)
            // orthogonally adjacent squares and WAITS for a CLIENT_SWOOP naming one — there is no
            // decline, unlike BB2025's optional skill offer. Coordinate-sorted candidate list plus a
            // single actionRng pick, exactly like every other target choice (AGENT_CONTRACT §6);
            // ParityRunner.sendSwoopTarget mirrors this list, order and draw.
            Some(AgentPrompt::SwoopTarget { player_id, squares }) => {
                let mut squares = squares.clone();
                squares.sort_by_key(|c| (c.x, c.y));
                if squares.is_empty() {
                    // Java would sit here forever; nothing legal means nothing to send.
                    Action::EndTurn
                } else {
                    let idx = self.pick_action(squares.len());
                    let target = squares[idx];
                    let is_home = gs.game.team_home.player(player_id).is_some();
                    let cmd_coord = if is_home { target } else { target.transform() };
                    Action::Swoop { coord: cmd_coord }
                }
            }
            Some(AgentPrompt::FollowUp { .. }) => {
                Action::FollowUp { follow_up: false }
            }
            // Block die selection: AGENT_CONTRACT §7/§8 — ALWAYS pick index 0, deterministically,
            // consuming 0 rng. Java ParityRunner BLOCK_ROLL = `sendBlockChoice(0)` (no decisionRng).
            // The old code random-sampled via pick(), consuming a spurious decision_rng draw that
            // desynced the stream for the next player pick (even a 1-die block drew a call).
            Some(AgentPrompt::BlockChoice { .. }) => {
                Action::BlockChoice { die_index: 0, target_id: None }
            }
            // Block choice with re-roll properties (BB2025): also index 0, 0 rng — Java
            // BLOCK_ROLL_PROPERTIES = `sendBlockChoice(0)`. The old code drew pick_bool (spurious).
            Some(AgentPrompt::BlockChoiceProperties { .. }) => {
                Action::BlockChoice { die_index: 0, target_id: None }
            }
            // Re-roll offer: AGENT_CONTRACT §7 — ALWAYS DECLINE, deterministically, 0 rng. Java
            // ParityRunner RE_ROLL / RE_ROLL_PROPERTIES = sendUseReRoll(action, null) (decline, no
            // decisionRng, no extra game die). The old code random-sampled via pick_bool, which both
            // sometimes USED a team reroll (extra dodge/etc. die → wrong injury) and drew a spurious
            // decision_rng call desyncing later picks.
            Some(AgentPrompt::ReRollOffer { .. }) =>
                Action::UseReRoll { use_reroll: false },
            // Dump Off is OPTIONAL and DECLINED by the parity harness: a blocked/blitzed ball-carrier
            // does NOT throw a Quick Pass. Java's ParityRunner sends no "use" for the DEFENDER_ACTION
            // dump-off dialog (RandomStrategy no-op), so the carrier keeps the ball and the block
            // proceeds. Rust previously answered every SkillUse with use_skill:true → it "used" dump-off
            // but then stalled (the dump-off pass was never driven; dark_elf seed 55 i=139: no i=140,
            // rust=None). Decline dump-off (use_skill:false) so the block proceeds like Java.
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "DumpOff" =>
                Action::UseSkill { skill_id: SkillId::DumpOff, use_skill: false },
            // Animal Savagery lash-out-against-opponents (Primal Savagery) is OPTIONAL: when the
            // confusion roll fails and an opponent is adjacent, Java offers a SKILL_USE to lash out
            // at that opponent instead of a team-mate. DECLINE it so both engines take the mandatory
            // team-mate branch (handled by the ANIMAL_SAVAGERY PlayerChoice arm below). The generic
            // "always use" arm would send skill_id=Block, which the AS step's PrimalSavagery-gated
            // handler ignores → the step would end with no lash-out at all. Java's ParityRunner
            // declines this SKILL_USE the same way it declines DumpOff.
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "PrimalSavagery" =>
                Action::UseSkill { skill_id: SkillId::PrimalSavagery, use_skill: false },
            // Hit And Run (BB2025): Java ParityRunner answers the SKILL_USE dialog with
            // always-use (contract §7). StepEndBlocking dispatches by the SENT skill's
            // canMoveAfterBlock property, so the generic Block placeholder would be ignored
            // and the dialog would refire forever — echo the real skill.
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "HitAndRun" =>
                Action::UseSkill { skill_id: SkillId::HitAndRun, use_skill: true },
            // Swoop (BB2025 TTM): DECLINE, like DumpOff/PrimalSavagery/SafePairOfHands. Using Swoop
            // opens a CLIENT_SWOOP target dialog that the parity harness (ParityRunner) has no
            // handler for → the SWOOP step gets STUCK and the game force-ends (goblin seed 3 i=194).
            // Declining lands the thrown player normally; ParityRunner declines Swoop identically.
            // Quick Bite (BB2020/BB2025): Java ParityRunner answers the SKILL_USE dialog with
            // always-use (contract §7, and QuickBite is not in its decline list). StepQuickBite
            // dispatches on the SENT skill's canAttackOpponentForBallAfterCatch property, so the
            // generic Block placeholder below would be ignored and the dialog would refire forever
            // — echo the real skill, exactly like the HitAndRun arm above.
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "QuickBite" =>
                Action::UseSkill { skill_id: SkillId::QuickBite, use_skill: true },
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "Swoop" =>
                Action::UseSkill { skill_id: SkillId::Swoop, use_skill: false },
            // Safe Pair of Hands (BB2020/BB2025): DECLINE, like DumpOff/PrimalSavagery/Swoop —
            // ParityRunner's SKILL_USE decline list names all four (using it enters TurnMode.
            // SAFE_PAIR_OF_HANDS → a PLACE_BALL coach dialog the harness cannot drive). The
            // generic always-use arm below sends the Block placeholder, which StepPlaceBall's
            // property gate ignores → the prompt refired forever and the game aborted on
            // NO_PROGRESS (chaos_pact bb2020 random seeds 9/23/25/55/83/94). The step routes the
            // DECLINED answer to leave(): the ball scatters in both engines.
            Some(AgentPrompt::SkillUse { skill_name, .. }) if skill_name == "SafePairOfHands" =>
                Action::UseSkill { skill_id: SkillId::SafePairOfHands, use_skill: false },
            // Skill use: AGENT_CONTRACT §7 — ALWAYS use, deterministically, 0 rng. Java ParityRunner
            // SKILL_USE = `sendUseSkill(skill, true, playerId)` (no decisionRng). The old code
            // random-sampled via pick_bool (spurious draw + wrong choice → decision-stream desync).
            // skill_id=Block is a placeholder (engine identifies the skill from step state).
            Some(AgentPrompt::SkillUse { .. }) =>
                Action::UseSkill { skill_id: SkillId::Block, use_skill: true },
            // Piling On: AGENT_CONTRACT §7 — always use, 0 rng (Java has no separate PILING_ON case;
            // treated as SKILL_USE = always use). Was random-sampled via pick_bool.
            Some(AgentPrompt::PilingOn { .. }) =>
                Action::UseSkill { skill_id: SkillId::Block, use_skill: true },
            // Apothecary: AGENT_CONTRACT §7 — DECLINE, deterministically, 0 rng. Java
            // APOTHECARY_CHOICE = sendApothecaryChoice(..., playerStateOld) i.e. keep the original
            // result (decline the apothecary). Was random-sampled via pick_bool (spurious draw).
            Some(AgentPrompt::ApothecaryChoice { player_id, .. }) =>
                Action::UseApothecary { player_id: player_id.clone(), use_apothecary: false },
            // Java harness `ParityRunner.handleDialog` case USE_APOTHECARY:
            //     comm.sendUseApothecary(apo.getPlayerId(), false, apoType, apo.getSeriousInjury());
            // i.e. it DECLINES, naming the injured player. Answering `Acknowledge` instead left
            // `StepApothecary` in APOTHECARY_STATUS `WAIT_FOR_APOTHECARY_USE` - a status its main
            // switch has no arm for - so the step fell through and the injury was never applied:
            // an Animal Savagery lash-out that KO'd its victim in Java left it STANDING in Rust
            // (underworld bb2020 seed 2: Java KOs `h02` at i=56, Rust never does).
            Some(AgentPrompt::UseApothecary { player_id, .. }) =>
                Action::UseApothecary { player_id: player_id.clone(), use_apothecary: false },
            // Block target asked for mid-sequence: `StepInitBlocking` reached with no
            // `blockDefenderId`. This only happens on the sequence `StepEndBlocking` re-pushes after
            // a failed Blood Lust, and the HARNESS never answers it: `INIT_BLOCKING` has no case in
            // `ParityRunner.handleStep`, so it falls to
            //     default: UNHANDLED_STEP: INIT_BLOCKING
            //              MatchRunner.inject(new ClientCommandEndTurn(game.getTurnMode(), null));
            // — the turn simply ENDS, with no actionRng draw and no block. (Verified: the vampire
            // seed-1 run logs exactly `UNHANDLED_STEP: INIT_BLOCKING`, and Java consumes ONE die at
            // i=100 before play passes to the other team at i=101.)
            // Picking a target here instead would roll block dice Java never rolls.
            // Java: StepSelectBlitzTarget's target wait. Candidates arrive coordinate-sorted
            // (ParityRunner.pickBlockTarget order); answer with exactly ONE actionRng draw so the
            // stream position matches the harness.
            //
            // Empty DOES happen, and the earlier "cannot happen - the step skips instead" note was
            // wrong: the engine shows this dialog whenever ANY in-bounds opponent can be blocked,
            // while these candidates are only the ADJACENT ones, so a blitzer with no neighbour
            // gets an empty list. ParityRunner.sendBlitzTargetSelection answers exactly that case
            // with ClientCommandEndTurn ("BLITZ_TARGET_NONE ... ending turn for acting player"),
            // and it must be EndTurn rather than EndPlayerAction - ending only the action left
            // Rust playing on with the rest of the team while Java's turn was over.
            Some(AgentPrompt::BlitzTarget { eligible_players, .. }) => {
                if eligible_players.is_empty() { return Action::EndTurn; }
                let idx = self.pick_action(eligible_players.len());
                Action::SelectPlayer { player_id: eligible_players[idx].clone() }
            }
            Some(AgentPrompt::BlockTarget { .. }) => Action::EndTurn,
            // Blastin' target wait: coordinate-sorted candidates, single actionRng pick →
            // SelectPlayer (CLIENT_TARGET_SELECTED). Empty list → EndTurn (client END_MOVE).
            Some(AgentPrompt::BlastinTarget { candidates, .. }) => {
                if candidates.is_empty() {
                    Action::EndTurn
                } else {
                    let mut sorted: Vec<&String> = candidates.iter().collect();
                    sorted.sort_by_key(|pid| {
                        gs.game.field_model.player_coordinate(pid)
                            .map(|c| (c.x, c.y)).unwrap_or((99, 99))
                    });
                    let idx = self.pick_action(sorted.len());
                    Action::SelectPlayer { player_id: sorted[idx].clone() }
                }
            }
            // Interception: pick a candidate, coordinate-sorted, with ONE actionRng draw
            // (AGENT_CONTRACT §6). `ParityRunner.sendInterceptorChoice` mirrors this list, order and
            // draw. The candidate set comes from the ENGINE's own `UtilPassing.findInterceptors` on
            // both sides, so the harness cannot drift from the engine it is testing — the mistake
            // the BB2020 Throw-Team-Mate campaign made with `canBeThrown`.
            // Sorted by COORDINATE, never by id: the two engines' player ids differ
            // (`away_03` vs `teamHighElfParity20Away3`), so an id sort would not agree.
            Some(AgentPrompt::Interception { candidates, .. }) => {
                let mut ids: Vec<String> = candidates.clone();
                ids.sort_by_key(|id| {
                    let c = gs.game.field_model.player_coordinate(id);
                    (c.map(|c| c.x).unwrap_or(i32::MAX), c.map(|c| c.y).unwrap_or(i32::MAX))
                });
                if ids.is_empty() {
                    Action::Intercept { attempt: false }
                } else {
                    let idx = self.pick_action(ids.len());
                    Action::SelectPlayer { player_id: ids[idx].clone() }
                }
            }
            // Touchback: Java ParityRunner picks the receiving player NEAREST to the fixed kick-from
            // square (13,8) by squared distance, iterating team order so the first candidate wins ties.
            // This is fully deterministic — NO decisionRng draw. The previous random pick both chose the
            // wrong player and consumed a spurious decision_rng draw, desyncing every later pick this half.
            Some(AgentPrompt::Touchback { eligible_players }) => {
                if eligible_players.is_empty() {
                    return Action::Acknowledge;
                }
                let kick_from = FieldCoordinate::new(13, 8);
                let best = eligible_players.iter()
                    .min_by_key(|(_, c)| {
                        let dx = c.x - kick_from.x;
                        let dy = c.y - kick_from.y;
                        dx * dx + dy * dy
                    })
                    .unwrap();
                Action::Touchback { player_id: best.0.clone() }
            }
            // Argue the call: AGENT_CONTRACT §7 — ALWAYS argue, deterministically, 0 rng. Java
            // ARGUE_THE_CALL always sends ClientCommandArgueTheCall(firstPlayer). Was random-sampled
            // via pick_bool (spurious draw + wrong choice).
            Some(AgentPrompt::ArgueTheCall { .. }) =>
                Action::ArgueTheCall { argue: true },
            // Player choice: pick uniformly from eligible sorted by PlayerId — 1 decision_rng call.
            // Synced with Java ParityRunner PLAYER_CHOICE dialog case.
            // Java ParityRunner (PLAYER_CHOICE handler) declines EVERY PlayerChoiceMode dialog with
            // an empty selection — sending `new Player[0]` and drawing NO rng — the sole exception
            // being MVP, which reaches the agent through a separate SelectPlayer prompt, not here.
            // So every `AgentPrompt::PlayerChoice` (Shadowing, Tentacles, Diving Tackle, Animal
            // Savagery, Pile Driver, Wisdom, …) must decline: an empty `player_id`, and crucially
            // no `pick()` — the previous code both selected a player AND consumed a decision_rng draw
            // Java never makes, desyncing the stream (e.g. a Shadowing roll that Java skips entirely).
            // ANIMAL_SAVAGERY is the ONE mandatory PlayerChoice: when the confusion roll fails and
            // there are ≥2 adjacent team-mates, Java shows DialogPlayerChoiceParameter(ANIMAL_SAVAGERY,
            // …, min=1, max=1) — the coach MUST pick exactly one team-mate to lash out at. Declining
            // with an empty selection (the generic path below) is invalid → Java re-fires the dialog
            // → STUCK_STEP, while Rust's AS step, never receiving Action::PlayerChoice, silently skips
            // the mandatory block. Pick the eligible team-mate with the MIN (x,y) board coordinate —
            // engine-agnostic (Rust ids `home_02` ≠ Java `teamRenegadesParityHome2`, but board coords
            // match) so both engines lash out at the SAME team-mate and the shared block dice align.
            // Java ParityRunner's PLAYER_CHOICE handler mirrors this exact min-(x,y) pick.
            // Multi-block reroll window (DialogReRollForTargetsParameter): DECLINE, exactly
            // like the plain RE_ROLL dialog — ParityRunner injects
            // ClientCommandUseReRollForTarget(action, null, null). 0 rng consumed either side.
            Some(AgentPrompt::ReRollForTargets { re_rolled_action, .. }) => {
                Action::UseReRollForTarget {
                    re_rolled_action: Some(re_rolled_action.clone()),
                    re_roll_source: None,
                    target_id: None,
                }
            }
            // MULTIPLE_BLOCK target window: pick TWO distinct targets from the
            // coordinate-sorted adjacent blockables — first idx % N, second idx % (N-1) —
            // exactly ParityRunner's sendSynchronousMultiBlock. With fewer than two targets
            // (both KO'd since the turn-start snapshot) deselect like any stale declaration.
            Some(AgentPrompt::MultiBlockTargets { player_id: blocker_id, eligible_players }) => {
                if eligible_players.len() < 2 {
                    return Action::EndPlayerAction;
                }
                let mut pool = eligible_players.clone();
                let i1 = self.pick_action(pool.len());
                let d1 = pool.remove(i1);
                let i2 = self.pick_action(pool.len());
                let d2 = pool.remove(i2);
                // Java's SynchronousMultiBlockLogicModule offers the STAB alternative exactly
                // when the acting player has `providesMultipleBlockAlternative` (registered by
                // Stab in every edition); otherwise it auto-selects BlockKind.BLOCK. Use that
                // ENGINE property rather than an invented rule, and take the alternative for the
                // FIRST-drawn target only - that way one multiblock exercises BOTH the block group
                // and the stab group. Deterministic: NO extra actionRng draw, because ParityRunner
                // already spends exactly two here and a third would desync the stream.
                // The BLOCKER is the prompt's own player_id - mirroring ParityRunner's
                // sendSynchronousMultiBlock(game, gameState, playerId) parameter. acting_player is
                // not yet committed at this continuation prompt, so reading it here found nobody
                // and the stab alternative was never offered.
                let can_stab = gs.game.player(blocker_id)
                    .map(|p| p.has_skill_property(
                        ffb_model::model::property::named_properties::NamedProperties::PROVIDES_MULTIPLE_BLOCK_ALTERNATIVE))
                    .unwrap_or(false);
                let kind1 = if can_stab {
                    ffb_model::model::block_kind::BlockKind::STAB
                } else {
                    ffb_model::model::block_kind::BlockKind::BLOCK
                };
                Action::MultiBlock {
                    // ParityRunner: new BlockTarget(dN.getId(), kind, fm.getPlayerState(dN)).
                    targets: vec![
                        ffb_model::model::block_target::BlockTarget::new(d1.clone(), kind1,
                            gs.game.field_model.player_state(&d1)),
                        ffb_model::model::block_target::BlockTarget::new(d2.clone(), ffb_model::model::block_kind::BlockKind::BLOCK,
                            gs.game.field_model.player_state(&d2)),
                    ],
                }
            }
            // BLACK_INK is the second mandatory PlayerChoice: Kiroth's auto-gaze picks its
            // victim with the same engine-agnostic min-(x,y) rule (ParityRunner's PLAYER_CHOICE
            // handler mirrors it), and the step consumes a plain Action::SelectPlayer.
            // Raiding Party team-mate choice (multi-eligible only; the step auto-picks a lone
            // candidate like Java). Contract with ParityRunner: single actionRng pick over the
            // dialog's list in the step's given order (Java's findPlayers = team nr order).
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "RAIDING_PARTY" =>
            {
                let idx = self.pick_action(eligible_players.len().max(1));
                Action::SelectPlayer {
                    player_id: eligible_players.get(idx).cloned().unwrap_or_default(),
                }
            }
            // "Excuse Me, Are You a Zoat?" gaze-target choice. ParityRunner has no arm for this
            // dialog either, so without one BOTH sides would fall to a default — Java's being the
            // NON-SEEDED RandomStrategy. Coordinate-sort then a single actionRng pick, matching
            // the harness arm; board coordinates are engine-agnostic.
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "AUTO_GAZE_ZOAT" =>
            {
                let mut cands: Vec<String> = eligible_players.clone();
                cands.sort_by_key(|pid| {
                    gs.game.field_model.player_coordinate(pid)
                        .map(|c| (c.x, c.y))
                        .unwrap_or((i32::MAX, i32::MAX))
                });
                if cands.is_empty() {
                    Action::SelectPlayer { player_id: String::new() }
                } else {
                    let idx = self.pick_action(cands.len());
                    Action::SelectPlayer { player_id: cands[idx].clone() }
                }
            }
            // Wisdom of the White Dwarf team-mate choice. MANDATORY (Java's dialog is built with
            // minSelects = 1), so an empty selection re-fires it forever. StepWisdomOfTheWhiteDwarf
            // builds `wisePlayers` from UtilPlayer.findStandingOrPronePlayers, whose order is not a
            // documented contract, so COORDINATE-SORT before the single actionRng pick — exactly
            // what ParityRunner's WISDOM arm does.
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "WISDOM" =>
            {
                let mut cands: Vec<String> = eligible_players.clone();
                cands.sort_by_key(|pid| {
                    gs.game.field_model.player_coordinate(pid)
                        .map(|c| (c.x, c.y))
                        .unwrap_or((i32::MAX, i32::MAX))
                });
                if cands.is_empty() {
                    Action::SelectPlayer { player_id: String::new() }
                } else {
                    let idx = self.pick_action(cands.len());
                    Action::SelectPlayer { player_id: cands[idx].clone() }
                }
            }
            // Furious Outburst stab-target choice. UNLIKE the other star dialogs the list order
            // is NOT a contract here: Java's StepInitFuriousOutburst collects `eligiblePlayers`
            // into a HashSet and hands `foundPlayers.toArray(..)` to the dialog, so its order is
            // identity-hash order and differs run to run. COORDINATE-SORT first, then a single
            // actionRng pick — board coordinates are engine-agnostic, so ParityRunner's
            // FURIOUS_OUTBURST arm lands on the same target.
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "FURIOUS_OUTBURST" =>
            {
                let mut cands: Vec<String> = eligible_players.clone();
                cands.sort_by_key(|pid| {
                    gs.game.field_model.player_coordinate(pid)
                        .map(|c| (c.x, c.y))
                        .unwrap_or((i32::MAX, i32::MAX))
                });
                if cands.is_empty() {
                    Action::SelectPlayer { player_id: String::new() }
                } else {
                    let idx = self.pick_action(cands.len());
                    Action::SelectPlayer { player_id: cands[idx].clone() }
                }
            }
            // Baleful Hex target choice: single actionRng pick over the dialog's list in
            // the step's given order (Java's findPlayers = opponent team nr order).
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "BALEFUL_HEX" =>
            {
                let idx = self.pick_action(eligible_players.len().max(1));
                Action::SelectPlayer {
                    player_id: eligible_players.get(idx).cloned().unwrap_or_default(),
                }
            }
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "BLACK_INK" =>
            {
                let best = eligible_players.iter().min_by_key(|pid| {
                    gs.game
                        .field_model
                        .player_coordinate(pid)
                        .map(|c| (c.x, c.y))
                        .unwrap_or((i32::MAX, i32::MAX))
                });
                Action::SelectPlayer {
                    player_id: best.cloned().unwrap_or_default(),
                }
            }
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "ANIMAL_SAVAGERY" =>
            {
                let best = eligible_players.iter().min_by_key(|pid| {
                    gs.game
                        .field_model
                        .player_coordinate(pid)
                        .map(|c| (c.x, c.y))
                        .unwrap_or((i32::MAX, i32::MAX))
                });
                match best {
                    Some(pid) => Action::PlayerChoice {
                        player_id: Some(pid.clone()),
                        player_ids: eligible_players.clone(),
                        mode: "ANIMAL_SAVAGERY".into(),
                    },
                    None => Action::SelectPlayer { player_id: String::new() },
                }
            }
            // The three BB2020 "Prayer to Nuffle" player choices (Iron Man, Knuckle Dusters,
            // Blessed Statue of Nuffle) are MANDATORY: Java declares their PlayerChoiceMode
            // non-declinable, `SelectPlayerPrayerHandler.applySelection` dereferences the chosen
            // player, and StepPrayer re-runs until it gets one. Pick the eligible player with the
            // LOWEST shirt number: `nr` comes from the same team data in both engines, and unlike
            // a board coordinate it is also well-defined for the RESERVE players this dialog
            // offers during START_GAME. ParityRunner's PLAYER_CHOICE arm mirrors this exact rule.
            Some(AgentPrompt::PlayerChoice { eligible_players, reason, .. })
                if reason == "IRON_MAN"
                    || reason == "KNUCKLE_DUSTERS"
                    || reason == "BLESSED_STATUE_OF_NUFFLE" =>
            {
                let best = eligible_players
                    .iter()
                    .min_by_key(|pid| gs.game.player(pid).map(|p| p.nr).unwrap_or(i32::MAX));
                match best {
                    Some(pid) => Action::PlayerChoice {
                        player_id: Some(pid.clone()),
                        player_ids: eligible_players.clone(),
                        mode: reason.clone(),
                    },
                    None => Action::SelectPlayer { player_id: String::new() },
                }
            }
            Some(AgentPrompt::PlayerChoice { .. }) => {
                Action::SelectPlayer { player_id: String::new() }
            }
            // Blood Lust (vampire failed the roll): keep the declared action rather than switching to
            // feed — deterministic, NO rng. Java's RandomStrategy uses an unseeded Random here, so
            // ParityRunner is given a matching deterministic BLOODLUST_ACTION handler (change=false).
            Some(AgentPrompt::BloodlustAction { .. }) =>
                Action::BloodlustAction { change: false },
            // Select weather: pick uniformly from options — 1 decision_rng call.
            Some(AgentPrompt::SelectWeather { options }) => {
                if options.is_empty() {
                    return Action::Acknowledge;
                }
                let idx = self.pick(options.len());
                Action::SelectWeather { weather: options[idx] }
            }
            // Hit And Run move window. Java's StepHitAndRun publishes the eligible squares as
            // MoveSquares and waits for a CLIENT_FIELD_COORDINATE naming one (CLIENT_END_TURN is
            // its abort); ParityRunner.sendHitAndRunTarget mirrors this list, order and single
            // actionRng pick (AGENT_CONTRACT §6). Both harnesses used to abort here, which is why
            // the `HitAndRun` step never executed while the matrices stayed green.
            // Raiding Party target square: MoveSquares published by StepRaidingParty; same
            // contract as HitAndRun/Punt — coordinate-sorted, single actionRng pick, CANONICAL
            // coordinate (the step stores it verbatim; ParityRunner sends the mirrored view and
            // Java un-mirrors it).
            // Furious Outburst teleport squares (FIRST_MOVE and SECOND_MOVE both use this
            // prompt): StepFirstMove/SecondMoveFuriousOutburst publish their eligible squares as
            // MoveSquares and wait for a CLIENT_FIELD_COORDINATE naming one. Same contract as
            // RaidingParty below — coordinate-sorted, single actionRng pick, CANONICAL coordinate
            // (ParityRunner.sendFuriousOutburstSquare sends the mirrored view and Java un-mirrors).
            // NOTE the abort differs from RaidingParty: Java's step has no CLIENT_END_TURN handler,
            // only a null-action CLIENT_ACTING_PLAYER, so an empty list ends the PLAYER ACTION.
            Some(AgentPrompt::FuriousOutburstSquare { player_id, squares }) => {
                let mut squares = squares.clone();
                squares.sort_by_key(|c| (c.x, c.y));
                if squares.is_empty() {
                    Action::EndPlayerAction
                } else {
                    let idx = self.pick_action(squares.len());
                    let _ = player_id;
                    Action::FuriousOutburstSquare { coord: squares[idx] }
                }
            }
            Some(AgentPrompt::RaidingParty { player_id, squares }) => {
                let mut squares = squares.clone();
                squares.sort_by_key(|c| (c.x, c.y));
                if squares.is_empty() {
                    Action::EndTurn
                } else {
                    let idx = self.pick_action(squares.len());
                    let _ = player_id;
                    Action::RaidingPartyTarget { coord: squares[idx] }
                }
            }
            Some(AgentPrompt::HitAndRun { player_id, squares }) => {
                let mut squares = squares.clone();
                squares.sort_by_key(|c| (c.x, c.y));
                if squares.is_empty() {
                    Action::EndTurn
                } else {
                    let idx = self.pick_action(squares.len());
                    // CANONICAL coordinate, deliberately NOT mirrored for the away coach. Java's
                    // StepHitAndRun un-mirrors what the away client sends
                    // (checkCommandIsFromHomePlayer -> transform), so ParityRunner sends the mirrored
                    // view and Java converts back; Rust's twin stores `coord` verbatim, so mirroring
                    // here would land the player on the reflected square (away_03 at 11,7 instead of
                    // Java's 14,7 — amazon bb2020 seed 1 i=2).
                    let _ = player_id;
                    Action::HitAndRun { coord: Some(squares[idx]) }
                }
            }
            // Punt target window. Java's StepInitPunt publishes the legal punt squares as
            // MoveSquares and waits for a CLIENT_FIELD_COORDINATE naming one; ParityRunner.sendPunt
            // mirrors this list, order and single actionRng pick (AGENT_CONTRACT §6). Both harnesses
            // used to abort here — Rust with EndTurn, Java with its UNHANDLED_STEP default — which
            // is why the whole Punt family (InitPunt, PuntDirection, PuntDistance, EndPunt) never
            // executed while the matrices stayed green.
            Some(AgentPrompt::PuntTarget { player_id, squares }) => {
                let mut squares = squares.clone();
                squares.sort_by_key(|c| (c.x, c.y));
                if squares.is_empty() {
                    // Nothing legal to punt to: Java's step falls through to its end label.
                    Action::EndTurn
                } else {
                    let idx = self.pick_action(squares.len());
                    let target = squares[idx];
                    let is_home = gs.game.team_home.player(player_id).is_some();
                    Action::Punt { coord: if is_home { target } else { target.transform() } }
                }
            }
            Some(AgentPrompt::TricksterMove { squares, .. }) => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let idx = self.pick_action(squares.len());
                Action::TricksterMove { coord: squares[idx] }
            }
            // Java harness: `ParityRunner` has no SELECT_SKILL case, so the dialog falls through to
            // `UNHANDLED_DIALOG` -> `RandomStrategy.respondToDialog`:
            //     case SELECT_SKILL: comm.sendSkillSelection(ss.getPlayerId(), skills.get(0));
            // i.e. the FIRST entry of the list Java built, which `IntensiveTrainingHandler` sorted
            // with `Comparator.comparing(Skill::getName)`. Deterministic, so it consumes NO decision
            // RNG - the old arm burned one call and answered `Acknowledge`, which the step ignores.
            // Sorting here rather than trusting the prompt's order keeps the answer correct however
            // the prompt groups the skills by category.
            Some(AgentPrompt::SelectSkill { skill_ids, .. }) => {
                let rules = gs.game.rules;
                // The prompt carries skills as u16 discriminants; `SkillFactory` is the registry
                // that can turn them back into `SkillId`s (Java: `SkillFactory.getSkills()`).
                let factory = ffb_model::factory::skill_factory::SkillFactory::new();
                let all: Vec<SkillId> = factory.get_skills().collect();
                // The list already arrives name-sorted from Java, so index 0 IS the answer;
                // min-by-name is kept as the explicit contract so a differently-ordered call
                // site can never silently change which skill is taken.
                let first = skill_ids
                    .iter()
                    .filter_map(|id| all.iter().copied().find(|s| *s as u16 == *id))
                    .min_by_key(|s| s.category_and_name_for(rules).1);
                match first {
                    Some(skill_id) => Action::SelectSkill { skill_id },
                    None => Action::Acknowledge,
                }
            }
            // Team setup: deterministic canonical formation, 0 RNG consumed — 1:1 with Java
            // ParityRunner's `resetCurrentTeam`/`placeReserves` (called directly per StepId,
            // bypassing the dialog entirely on the Java side; here the engine models it as a
            // real dialog, so the agent drip-feeds one PlacePlayer per prompt then confirms).
            Some(AgentPrompt::TeamSetup { team_id, .. }) =>
                canonical_setup_action(&gs.game, team_id),
            // Interactive kickoff events (Quick Snap / Solid Defence / High Kick): decline the
            // optional placements — 1:1 with Java ParityRunner's EndTurn at APPLY_KICKOFF_RESULT.
            // 0 RNG consumed on either side.
            Some(AgentPrompt::KickoffEventPlacement { .. }) => Action::EndTurn,
            // Inducement / pre-game: always decline / acknowledge with no RNG consumed.
            Some(AgentPrompt::BuyInducements { team_id, .. }) =>
                Action::BuyInducements { home: *team_id == gs.game.team_home.id, purchases: vec![] },
            Some(AgentPrompt::BuyPrayersAndInducements { team_id, .. }) =>
                Action::BuyInducements { home: *team_id == gs.game.team_home.id, purchases: vec![] },
            // AgentPrompt::SwarmingPlayers is (mis)named — in the live engine it is emitted ONLY by
            // StepCatchScatterThrowIn::diving_catch as the DIVING CATCH declaration dialog (Java
            // DialogParameterDivingCatch), NOT by StepSwarming. The step advances its DivingCatchPhase
            // (AskHome→AskAway→Process) on Action::SelectPlayer; an EMPTY player_id declines (no catcher)
            // and still advances. The agent previously answered Acknowledge (and EndTurn) which the step
            // ignores → the prompt re-fired forever (slann seed 3 half-2 kickoff: NO_PROGRESS → rust=None).
            // Decline like Java's ParityRunner (same pattern as the dark_elf PlayerChoice decline).
            Some(AgentPrompt::SwarmingPlayers { .. }) => Action::SelectPlayer { player_id: String::new() },
            // Bomb re-throw window (TurnMode Bomb*). StepInitPassing.executeStep returns
            // immediately while the thrower is unset, so a decline CANNOT advance the step --
            // EndTurn and deselect both set their flag and are swallowed by that early return.
            // The bomb must actually be thrown. 1:1 with ParityRunner's INIT_PASSING case, which
            // calls the ordinary sendPassAction: same candidate rule (all on-pitch teammates,
            // coordinate-sorted), same single actionRng draw, and the same empty-list fallback of
            // 2 decisionRng draws for a random square.
            Some(AgentPrompt::BombRethrow { player_id }) => {
                let pid = player_id.clone();
                let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                let receivers = legal_pass_receivers(&gs.game, &pid, side);
                let coord = if receivers.is_empty() {
                    let x = self.pick(26) as i32;
                    let y = self.pick(14) as i32 + 1;
                    ffb_model::types::FieldCoordinate::new(x, y)
                } else {
                    let idx = self.pick_action(receivers.len());
                    gs.game.field_model.player_coordinate(&receivers[idx])
                        .expect("legal_pass_receivers only returns on-pitch players")
                };
                Action::Pass { coord }
            }
            // Confirm-only and informational prompts: single valid response, 0 RNG consumed.
            Some(AgentPrompt::KickoffReturn { .. })
            | Some(AgentPrompt::SetupError { .. })
            | Some(AgentPrompt::ConfirmEndAction { .. })
            | Some(AgentPrompt::InformationOkay { .. })
            | Some(AgentPrompt::StartGame)
            | Some(AgentPrompt::GameStatistics)
            | Some(AgentPrompt::DefenderAction { .. })
            | Some(AgentPrompt::PettyCash { .. })
            | Some(AgentPrompt::UseInducement { .. })
            | Some(AgentPrompt::WizardSpell { .. })
            | Some(AgentPrompt::BriberyAndCorruption { .. })
            | Some(AgentPrompt::ConcedeGame { .. })
            | Some(AgentPrompt::Journeymen { .. })
            | Some(AgentPrompt::SelectPosition { .. }) =>
                Action::Acknowledge,
            // Each remaining prompt is wired as its producing step lands in Phase D; the loud
            // failure here names exactly which handler is still missing.
            other => panic!("RandomAgent::act: no handler yet for prompt {other:?}"),
        }
    }
}

/// Convert a model-level `PlayerAction` (from `AgentPrompt`) back to the engine's
/// `PlayerActionChoice` (for `Action::ActivatePlayer`). Covers the lineman-reachable set.
pub(crate) fn player_action_to_pac(pa: &PlayerAction) -> PlayerActionChoice {
    match pa {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
        | PlayerAction::HandOverMove | PlayerAction::FoulMove | PlayerAction::GazeMove
        | PlayerAction::BlitzSelect | PlayerAction::KickTeamMateMove
        | PlayerAction::PuntMove => PlayerActionChoice::Move,
        PlayerAction::Block      => PlayerActionChoice::Block,
        PlayerAction::Blitz      => PlayerActionChoice::Blitz,
        PlayerAction::StandUp | PlayerAction::RemoveConfusion => PlayerActionChoice::StandUp,
        PlayerAction::StandUpBlitz => PlayerActionChoice::StandUpBlitz,
        PlayerAction::Foul       => PlayerActionChoice::Foul,
        PlayerAction::Pass | PlayerAction::DumpOff => PlayerActionChoice::Pass,
        PlayerAction::HailMaryPass => PlayerActionChoice::HailMaryPass,
        PlayerAction::RaidingParty => PlayerActionChoice::RaidingParty,
        PlayerAction::FuriousOutburst => PlayerActionChoice::FuriousOutburst,
        PlayerAction::ThrowKeg => PlayerActionChoice::ThrowKeg,
        PlayerAction::WisdomOfTheWhiteDwarf => PlayerActionChoice::WisdomOfTheWhiteDwarf,
        PlayerAction::AutoGazeZoat => PlayerActionChoice::AutoGazeZoat,
        PlayerAction::LookIntoMyEyes => PlayerActionChoice::LookIntoMyEyes,
        PlayerAction::BalefulHex => PlayerActionChoice::BalefulHex,
        PlayerAction::CatchOfTheDay => PlayerActionChoice::CatchOfTheDay,
        PlayerAction::ThenIStartedBlastin => PlayerActionChoice::ThenIStartedBlastin,
        PlayerAction::AllYouCanEat => PlayerActionChoice::AllYouCanEat,
        PlayerAction::Treacherous => PlayerActionChoice::Treacherous,
        PlayerAction::MultipleBlock => PlayerActionChoice::MultipleBlock,
        PlayerAction::BlackInk => PlayerActionChoice::BlackInk,
        PlayerAction::HandOver      => PlayerActionChoice::HandOff,
        PlayerAction::SecureTheBall => PlayerActionChoice::SecureTheBall,
        PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => PlayerActionChoice::ThrowTeamMate,
        PlayerAction::KickTeamMate => PlayerActionChoice::KickTeamMate,
        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::LookIntoMyEyes
        | PlayerAction::AutoGazeZoat => PlayerActionChoice::HypnoticGaze,
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => PlayerActionChoice::ThrowBomb,
        PlayerAction::Swoop => PlayerActionChoice::Swoop,
        PlayerAction::Punt => PlayerActionChoice::Punt,
        PlayerAction::BreatheFire => PlayerActionChoice::BreatheFire,
        PlayerAction::ProjectileVomit | PlayerAction::PutridRegurgitationMove
        | PlayerAction::PutridRegurgitationBlitz | PlayerAction::PutridRegurgitationBlock => PlayerActionChoice::ProjectileVomit,
        PlayerAction::Chainsaw | PlayerAction::Stab => PlayerActionChoice::Stab,
        // Skills that modify existing actions — treat as the underlying action type
        PlayerAction::MultipleBlock | PlayerAction::KickEmBlock => PlayerActionChoice::Block,
        PlayerAction::KickEmBlitz => PlayerActionChoice::Blitz,
        // Special actions with no direct PAC equivalent — default to Move
        PlayerAction::Treacherous
        | PlayerAction::RaidingParty | PlayerAction::MaximumCarnage | PlayerAction::BalefulHex
        | PlayerAction::AllYouCanEat | PlayerAction::BlackInk | PlayerAction::CatchOfTheDay
        | PlayerAction::ThenIStartedBlastin | PlayerAction::TheFlashingBlade
        | PlayerAction::ViciousVines | PlayerAction::Chomp
        | PlayerAction::Incorporeal | PlayerAction::Forgo => PlayerActionChoice::Move,
    }
}

#[cfg(test)]
mod furious_outburst_declaration_tests {
    use super::*;

    /// `is_handled_acting_action` mirrors `ParityRunner.isHandledActingAction`. Omitting a star
    /// action here makes the agent DECLARE it and then instantly deselect, so the mechanic never
    /// executes while the matrices stay green — the exact shape that kept Kick Team-Mate dead in
    /// every edition, and that kept Furious Outburst dead until wood_elf bb2025 seed 1 i=23.
    #[test]
    fn furious_outburst_is_a_handled_acting_action() {
        assert!(is_handled_acting_action(PlayerActionChoice::FuriousOutburst));
    }

    /// Same list, same trap: THROW_KEG is declared in two commands but is still a handled
    /// acting action — leaving it out would declare and instantly deselect every keg.
    #[test]
    fn throw_keg_is_a_handled_acting_action() {
        assert!(is_handled_acting_action(PlayerActionChoice::ThrowKeg));
    }

    #[test]
    fn wisdom_is_a_handled_acting_action() {
        assert!(is_handled_acting_action(PlayerActionChoice::WisdomOfTheWhiteDwarf));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::new_game;

    /// ParityRunner.sendBlitzTargetSelection answers an unanswerable blitz window with
    /// ClientCommandEndTurn ("BLITZ_TARGET_NONE ... ending turn for acting player"). The engine
    /// opens that window whenever ANY in-bounds opponent can be blocked, while these candidates
    /// are only the ADJACENT ones - so an empty list is routine, not impossible as the code once
    /// claimed. It must be EndTurn and NOT EndPlayerAction: ending only the action left Rust
    /// playing on with the rest of the team while Java's turn was already over (lineman bb2025
    /// seed 14, where Java flipped to the away team at step 11 and Rust activated home_09).
    #[test]
    fn blitz_target_prompt_with_no_candidates_ends_the_turn() {
        let mut gs = new_game(1);
        let mut agent = RandomAgent::new(1);
        gs.pending_prompt = Some(AgentPrompt::BlitzTarget {
            attacker_id: "home_01".into(),
            eligible_players: Vec::new(),
        });
        assert_eq!(agent.act(&gs), Action::EndTurn);
    }

    /// The non-empty case still spends exactly ONE actionRng draw, at the stream position the
    /// harness uses, and picks from the coordinate-sorted candidate list.
    #[test]
    fn blitz_target_prompt_picks_one_candidate() {
        let mut gs = new_game(1);
        let mut agent = RandomAgent::new(1);
        gs.pending_prompt = Some(AgentPrompt::BlitzTarget {
            attacker_id: "home_01".into(),
            eligible_players: vec!["away_01".into(), "away_02".into()],
        });
        let before = agent.action_rng_count;
        match agent.act(&gs) {
            Action::SelectPlayer { player_id } => {
                assert!(player_id == "away_01" || player_id == "away_02");
            }
            other => panic!("expected SelectPlayer, got {other:?}"),
        }
        assert_eq!(agent.action_rng_count, before + 1, "exactly one actionRng draw");
    }

    /// The full boundary loop (current_prompt → act → apply) drives the pregame to idle, and the
    /// agent's decision draws match a reference decision RNG seeded per the contract — validating
    /// the agent RNG contract on coin/receive FIRST, before rule prompts exist (plan risk item).
    #[test]
    fn pushback_picks_min_xy_square_deterministically() {
        // AGENT_CONTRACT §7 + Java ParityRunner.sendPushback: min x, ties by min y. Parity bug fix:
        // the agent used to random-index a sorted list (and consume a decisionRng call), landing on
        // the wrong square (e.g. (11,9) instead of (11,7)) and desyncing the decision stream.
        let squares = [
            FieldCoordinate::new(11, 9),
            FieldCoordinate::new(11, 8),
            FieldCoordinate::new(11, 7),
        ];
        assert_eq!(choose_pushback_square(&squares), Some(FieldCoordinate::new(11, 7)));
        // Order-independent + deterministic (pure function → no decisionRng consumed).
        let reordered = [
            FieldCoordinate::new(11, 7),
            FieldCoordinate::new(12, 6),
            FieldCoordinate::new(11, 9),
        ];
        assert_eq!(choose_pushback_square(&reordered), Some(FieldCoordinate::new(11, 7)));
        assert_eq!(choose_pushback_square(&[]), None);
    }

    /// Mirrors `ParityRunner.handleDialog` case USE_APOTHECARY:
    ///     `comm.sendUseApothecary(apo.getPlayerId(), false, apoType, apo.getSeriousInjury())`
    /// The agent must DECLINE and name the injured player. Answering `Acknowledge` (the old
    /// behaviour) left `StepApothecary` stuck in `WAIT_FOR_APOTHECARY_USE`, whose main switch has
    /// no arm, so the computed injury was silently discarded - an Animal Savagery lash-out that
    /// KO'd its victim in Java left it STANDING in Rust (underworld bb2020 seed 2).
    /// Hit and Run answers with a CANONICAL square, coordinate-sorted, one `actionRng` draw
    /// (AGENT_CONTRACT §6). Rust's `StepHitAndRun` stores `coord` verbatim, unlike
    /// `StepInitThrowTeamMate` which un-mirrors it — mirroring here put the away player on the
    /// reflected square (amazon bb2020 seed 1 i=2). Before this arm existed both harnesses aborted
    /// the window, so `StepHitAndRun` never executed while the matrices stayed green.
    #[test]
    fn hit_and_run_answers_a_sorted_canonical_square_for_either_coach() {
        for side in ["home_03", "away_03"] {
            let mut gs = new_game(1);
            let squares = vec![
                FieldCoordinate::new(14, 7),
                FieldCoordinate::new(11, 9),
                FieldCoordinate::new(11, 7),
            ];
            gs.pending_prompt = Some(AgentPrompt::HitAndRun {
                player_id: side.into(),
                squares: squares.clone(),
            });
            let mut agent = RandomAgent::new_parity(1);
            let before = agent.decision_rng_count;
            let action = agent.act(&gs);
            let mut sorted = squares.clone();
            sorted.sort_by_key(|c| (c.x, c.y));
            match action {
                Action::HitAndRun { coord: Some(c) } => {
                    assert!(sorted.contains(&c), "must pick from the published squares");
                    assert!(!sorted.iter().any(|s| s.transform() == c && !sorted.contains(&c)),
                        "the square is sent canonical, never mirrored");
                }
                other => panic!("expected HitAndRun, got {other:?}"),
            }
            assert_eq!(agent.decision_rng_count, before,
                "the square comes from actionRng, not the decision stream");
        }
    }

    /// An empty Hit and Run window ends the turn rather than sending a coordinate — Java's
    /// `StepHitAndRun` treats `CLIENT_END_TURN` as its abort.
    #[test]
    fn hit_and_run_with_no_squares_ends_the_turn() {
        let mut gs = new_game(1);
        gs.pending_prompt = Some(AgentPrompt::HitAndRun {
            player_id: "home_03".into(),
            squares: vec![],
        });
        let mut agent = RandomAgent::new_parity(1);
        assert!(matches!(agent.act(&gs), Action::EndTurn));
    }

    /// Punt answers with the MIRRORED square for the away coach: Java's `StepInitPunt` un-mirrors
    /// what the away client sends, so the harness must send the mirrored view (the opposite of
    /// `HitAndRun` above — always read the target step's `handle_command` first).
    #[test]
    fn punt_target_is_mirrored_for_the_away_coach_only() {
        let squares = vec![FieldCoordinate::new(13, 4), FieldCoordinate::new(12, 6)];
        let mut picks = vec![];
        for side in ["home_03", "away_03"] {
            let mut gs = new_game(1);
            let mut p = ffb_model::model::player::Player::default();
            p.id = side.into();
            if side.starts_with("home") { gs.game.team_home.players.push(p); }
            else { gs.game.team_away.players.push(p); }
            gs.pending_prompt = Some(AgentPrompt::PuntTarget {
                player_id: side.into(),
                squares: squares.clone(),
            });
            let mut agent = RandomAgent::new_parity(1);
            let before = agent.decision_rng_count;
            match agent.act(&gs) {
                Action::Punt { coord } => picks.push(coord),
                other => panic!("expected Punt, got {other:?}"),
            }
            assert_eq!(agent.decision_rng_count, before);
        }
        assert!(squares.contains(&picks[0]), "the home coach sends the canonical square");
        assert_eq!(picks[1], picks[0].transform(), "the away coach sends the mirrored view");
    }

    /// Both harnesses ATTEMPT the interception in lockstep: Rust answers `SelectPlayer`, Java's
    /// `ParityRunner.sendInterceptorChoice` picks from the same engine-computed candidate list with
    /// the same ordering and a single `actionRng` draw. The pick is COORDINATE-sorted, never
    /// id-sorted — the two engines' player ids differ.
    #[test]
    fn interception_picks_a_coordinate_sorted_candidate_with_one_action_draw() {
        let mut gs = new_game(1);
        for (id, x, y) in [("away_05", 14, 7), ("away_02", 11, 9), ("away_04", 11, 6)] {
            let mut p = ffb_model::model::player::Player::default();
            p.id = id.into();
            gs.game.team_away.players.push(p);
            gs.game.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
        }
        let candidates = vec!["away_05".to_string(), "away_02".to_string(), "away_04".to_string()];
        gs.pending_prompt = Some(AgentPrompt::Interception {
            player_id: candidates[0].clone(),
            target_number: 0,
            candidates: candidates.clone(),
        });
        let mut agent = RandomAgent::new_parity(1);
        let before = agent.decision_rng_count;
        match agent.act(&gs) {
            Action::SelectPlayer { player_id } => {
                assert!(candidates.contains(&player_id), "must pick a published candidate");
            }
            other => panic!("expected SelectPlayer, got {other:?}"),
        }
        assert_eq!(agent.decision_rng_count, before,
            "the candidate comes from actionRng, not the decision stream");
    }

    /// An empty candidate list declines, matching Java's `sendInterceptorChoice(null, null)`.
    #[test]
    fn interception_with_no_candidates_declines() {
        let mut gs = new_game(1);
        gs.pending_prompt = Some(AgentPrompt::Interception {
            player_id: String::new(),
            target_number: 0,
            candidates: vec![],
        });
        let mut agent = RandomAgent::new_parity(1);
        assert!(matches!(agent.act(&gs), Action::Intercept { attempt: false }));
    }

    #[test]
    fn use_apothecary_prompt_is_declined_naming_the_injured_player() {
        let mut gs = new_game(1);
        gs.pending_prompt = Some(AgentPrompt::UseApothecary {
            player_id: "home_03".into(),
            apothecary_type: "team".into(),
        });
        let mut agent = RandomAgent::new_parity(1);
        let before = agent.decision_rng_count;
        let action = agent.act(&gs);
        match action {
            Action::UseApothecary { player_id, use_apothecary } => {
                assert_eq!(player_id, "home_03");
                assert!(!use_apothecary, "the harness always declines");
            }
            other => panic!("expected a declined UseApothecary, got {other:?}"),
        }
        // Java sends the decline unconditionally - no coin flip, so no decision RNG is consumed.
        assert_eq!(agent.decision_rng_count, before);
    }

    /// Mirrors `RandomStrategy.respondToDialog` case SELECT_SKILL, which `ParityRunner` reaches via
    /// its `UNHANDLED_DIALOG` fallthrough: `comm.sendSkillSelection(ss.getPlayerId(), skills.get(0))`
    /// - the first entry of the name-sorted list `IntensiveTrainingHandler` built. The old arm burned
    /// a decision-RNG call and answered `Acknowledge`, which the step ignores, so the Intensive
    /// Training prayer granted nothing (lineman bb2020 seed 50).
    #[test]
    fn select_skill_prompt_answers_the_name_first_skill_with_no_rng() {
        let mut gs = new_game(1);
        // Offered OUT of name order to prove the min-by-name contract holds even if a call site
        // ever hands over an unsorted list (Java's two call sites both sort by name).
        gs.pending_prompt = Some(AgentPrompt::SelectSkill {
            player_id: "home_02".into(),
            skill_ids: vec![SkillId::Tackle as u16, SkillId::Block as u16, SkillId::Fend as u16],
            reason: "INTENSIVE_TRAINING".into(),
        });
        let mut agent = RandomAgent::new_parity(1);
        let before = agent.decision_rng_count;
        match agent.act(&gs) {
            Action::SelectSkill { skill_id } => assert_eq!(skill_id, SkillId::Block),
            other => panic!("expected SelectSkill(Block), got {other:?}"),
        }
        assert_eq!(agent.decision_rng_count, before, "Java's answer is deterministic");
    }

    #[test]
    fn random_agent_drives_pregame_with_contract_decision_rng() {
        let seed = 1u64;
        // Reference decision stream: coin guess, then receive — two pick_bool draws.
        let mut ref_dec = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        let exp_heads = ref_dec.next_u64() % 2 == 0;
        let exp_receive = ref_dec.next_u64() % 2 == 0;

        let mut gs = new_game(seed);
        gs.run_until_prompt();
        let mut agent = RandomAgent::new_parity(seed);

        // Drive exactly the 3 pregame actions (coin, receive, kick); stop before the first
        // ActivatePlayer so we test the pregame RNG contract in isolation.
        let mut actions = Vec::new();
        while gs.current_prompt().is_some() && actions.len() < 3 {
            let a = agent.act(&gs);
            actions.push(a.clone());
            gs.apply_action(a);
        }

        assert_eq!(actions.len(), 3, "pregame asks coin, receive, then KickBall");
        assert!(matches!(actions[0], Action::CoinChoice { heads } if heads == exp_heads));
        assert!(matches!(actions[1], Action::ReceiveChoice { receive } if receive == exp_receive));
        assert!(matches!(actions[2], Action::KickBall { .. }));
        // After KickBall the engine drives past the kickoff to the next coach prompt (with the
        // empty-roster test teams that is the touchback declaration or the first — empty —
        // ActivatePlayer). Either way the pregame decision phase is over.
        assert!(
            matches!(gs.current_prompt(),
                Some(AgentPrompt::Touchback { .. }) | Some(AgentPrompt::ActivatePlayer { .. })),
            "engine drove past the kickoff to a coach prompt, got {:?}", gs.current_prompt());
        // The agent's decision RNG must not touch the game-dice GameRng: coin/receive/kick draw from
        // decision_rng (4 draws: §2.1 coin, §2.2 receive, §2.3 kick x/y), while the game-dice count is
        // an engine-only, deterministic property of the seed. Assert determinism (a second identical
        // run rolls the same number of game dice) rather than a brittle hard-coded count.
        let game_dice = gs.rng.call_count;
        assert!(game_dice > 0, "engine rolled pregame + kickoff game dice");
        let mut gs2 = new_game(seed);
        gs2.run_until_prompt();
        let mut agent2 = RandomAgent::new_parity(seed);
        let mut n = 0;
        while gs2.current_prompt().is_some() && n < 3 {
            let a = agent2.act(&gs2);
            gs2.apply_action(a);
            n += 1;
        }
        assert_eq!(game_dice, gs2.rng.call_count,
            "game-dice stream is deterministic — the agent's decision RNG never perturbs it");
    }

    /// `ParityRunner.sendMoveAction` draws exactly ONE actionRng move target per activation. Rust
    /// pre-draws that target at activation for players that cannot use the normal StepInitMoving
    /// prompt path (PRONE — its Select-sequence negatrait may fail first; ROOTED — its StepMove is a
    /// no-op), storing it in `pending_move`; the Move-prompt handler must then reuse it and draw
    /// NOTHING further. This test pins that reuse contract.
    ///
    /// The pre-draw used to be stored only for PRONE players, so a ROOTED player's pre-draw was
    /// discarded and the Move prompt drew AGAIN — 3 actionRng calls where Java makes 2, permanently
    /// shifting the agent stream (wood_elf bb2016 seed 2 step 64: the rooted Treeman pre-drew (13,6),
    /// the square Java uses, then re-drew (11,7); first visible as home_10 moving to (3,7) rather
    /// than (5,7) at i=65). The *storing* half is covered end-to-end by the wood_elf parity run
    /// (81 -> 19 fails); this unit test guards the zero-extra-draw reuse.
    #[test]
    fn predrawn_move_square_is_reused_with_no_extra_draw() {
        let mut gs = new_game(1);
        gs.game.home_playing = true;
        let mut agent = RandomAgent::new_parity(1);

        let predrawn = FieldCoordinate::new(13, 6);
        agent.pending_move = Some(predrawn);
        let arc_before = agent.action_rng_count;

        gs.pending_prompt = Some(AgentPrompt::Move {
            player_id: "tree".into(),
            // A DIFFERENT candidate list: if the handler re-drew it could not return `predrawn`.
            squares: vec![FieldCoordinate::new(11, 6), FieldCoordinate::new(11, 7)],
        });

        match agent.act(&gs) {
            Action::Move { path } => assert_eq!(path, vec![predrawn],
                "the pre-drawn square is used verbatim, not re-picked from the prompt list"),
            other => panic!("expected Move, got {other:?}"),
        }
        assert_eq!(agent.action_rng_count, arc_before,
            "reusing the pre-drawn square must consume NO further actionRng");
        assert!(agent.pending_move.is_none(), "the pre-drawn square is consumed exactly once");
    }

    #[test]
    fn any_player_deselects_after_first_move() {
        // Stock ParityRunner INIT_MOVING always deselects after the first square — even a ball
        // carrier below MA does not keep moving (the old carrier-continue rule was mirrored
        // against a modified harness and is gone).
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        let mut gs = new_game(1);
        gs.game.team_home.players.push(Player {
            id: "carrier".into(), name: "carrier".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false, ..Default::default()
        });
        let coord = FieldCoordinate::new(9, 9);
        gs.game.field_model.set_player_coordinate("carrier", coord);
        gs.game.field_model.set_player_state("carrier", PlayerState::new(PS_STANDING));
        gs.game.field_model.ball_coordinate = Some(coord); // carrier holds the ball
        gs.game.field_model.ball_moving = false;
        gs.game.home_playing = true;
        gs.game.acting_player.set_player("carrier".into(), PlayerAction::Move);

        let squares = vec![FieldCoordinate::new(8, 8), FieldCoordinate::new(8, 9)];
        let mut agent = RandomAgent::new_parity(1);
        agent.moved_this_activation = true; // 2nd+ Move prompt of the activation

        // Below MA the carrier STILL deselects (no carrier-continue in the stock harness).
        gs.game.acting_player.current_move = 5;
        gs.pending_prompt = Some(AgentPrompt::Move { player_id: "carrier".into(), squares });
        assert!(matches!(agent.act(&gs), Action::EndPlayerAction), "one move per activation, then deselect");
    }

    #[test]
    fn move_pick_uses_full_neighbour_list_no_carrier_bias() {
        // A ball carrier's first move picks from ALL sorted neighbours (no advance filter). With a
        // fixed action_rng the chosen index is (draw % squares.len()) over the full list, mirroring
        // ParityRunner.sendMoveAction.
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        let mut gs = new_game(1);
        gs.game.team_home.players.push(Player {
            id: "carrier".into(), name: "carrier".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        });
        let coord = FieldCoordinate::new(13, 8);
        gs.game.field_model.set_player_coordinate("carrier", coord);
        gs.game.field_model.set_player_state("carrier", PlayerState::new(PS_STANDING));
        gs.game.field_model.ball_coordinate = Some(coord); // holds the ball
        gs.game.field_model.ball_moving = false;
        gs.game.home_playing = false; // away carrier — old bias would have filtered to x<13
        gs.game.acting_player.set_player("carrier".into(), PlayerAction::Move);

        // Full sorted neighbour set including non-advancing (x>=13) squares.
        let squares = vec![
            FieldCoordinate::new(12, 7), FieldCoordinate::new(12, 9),
            FieldCoordinate::new(13, 9), FieldCoordinate::new(14, 7),
            FieldCoordinate::new(14, 8), FieldCoordinate::new(14, 9),
        ];
        let mut agent = RandomAgent::new_parity(1);
        agent.moved_this_activation = false; // first Move prompt
        gs.pending_prompt = Some(AgentPrompt::Move { player_id: "carrier".into(), squares: squares.clone() });
        // The pick must be able to land on a non-advancing square: it indexes the full 6-list,
        // not a 2-element advancing subset. Assert the chosen target comes from the full list.
        match agent.act(&gs) {
            Action::Move { path } => {
                assert_eq!(path.len(), 1);
                assert!(squares.contains(&path[0]), "target must come from the full neighbour list");
            }
            other => panic!("expected a Move, got {other:?}"),
        }
    }

    #[test]
    fn prone_move_predraw_is_reused_without_second_action_rng_draw() {
        // Regression (docs/PARITY_TTM.md "FRONTIER (human)", the Ogre Bone-head case): a prone
        // (standing-up) player activated for Move pre-draws its move target at activation — mirroring
        // Java ParityRunner.sendMoveAction, which draws the move-target actionRng at phase-2, BEFORE
        // the Select-sequence negatrait roll. A Bone-head failure ends the activation before
        // StepInitMoving emits AgentPrompt::Move, yet Java has already drawn. Pre-drawing keeps the
        // stream aligned; when the Move prompt DOES arrive, the square is reused with NO second draw.
        let mut gs = new_game(1);
        let mut agent = RandomAgent::new_parity(1);
        let sq = FieldCoordinate::new(14, 6);
        agent.pending_move = Some(sq);
        agent.moved_this_activation = false;
        let arc_before = agent.action_rng_count;
        gs.pending_prompt = Some(AgentPrompt::Move {
            player_id: "away_01".into(),
            squares: vec![FieldCoordinate::new(13, 6), sq, FieldCoordinate::new(15, 6)],
        });
        match agent.act(&gs) {
            Action::Move { path } => assert_eq!(path, vec![sq], "reuses the pre-drawn square"),
            other => panic!("expected Move to the pre-drawn square, got {other:?}"),
        }
        assert_eq!(agent.action_rng_count, arc_before,
            "reusing the pre-drawn square must NOT draw a second action_rng");
        assert!(agent.pending_move.is_none(), "pending_move is consumed after reuse");
        // The follow-up Move prompt (same activation) deselects: one move per activation.
        gs.pending_prompt = Some(AgentPrompt::Move { player_id: "away_01".into(), squares: vec![sq] });
        assert!(matches!(agent.act(&gs), Action::EndPlayerAction),
            "the second Move prompt of the activation deselects");
    }

    #[test]
    fn parity_seeds_are_distinct_streams() {
        // Decision and action RNGs must diverge immediately (different seed XORs) so action
        // diversity never perturbs the Java-synced decision stream.
        let mut a = RandomAgent::new_parity(7);
        let d = a.decision_rng.next_u64();
        let act = a.action_rng.next_u64();
        assert_ne!(d, act);
    }

    #[test]
    fn new_parity_and_new_produce_different_decision_streams() {
        // new_parity applies XOR 0xDEAD_BEEF_CAFE_0001 to seed; new() uses seed directly.
        let seed = 42u64;
        let mut parity = RandomAgent::new_parity(seed);
        let mut plain = RandomAgent::new(seed);
        // The decision_rng streams must differ because the seeds differ.
        let parity_draw = parity.decision_rng.next_u64();
        let plain_draw = plain.decision_rng.next_u64();
        assert_ne!(parity_draw, plain_draw, "parity XOR makes decision RNG distinct from plain");
    }

    #[test]
    fn new_same_seed_produces_same_sequence() {
        let mut a1 = RandomAgent::new(99);
        let mut a2 = RandomAgent::new(99);
        for _ in 0..5 {
            assert_eq!(a1.decision_rng.next_u64(), a2.decision_rng.next_u64());
        }
    }

    #[test]
    fn pick_t2_activation_advances_decision_rng() {
        let mut a1 = RandomAgent::new(1);
        let mut a2 = RandomAgent::new(1);
        // Consume one draw on a1 via pick_t2_activation; a2 stays at its initial position.
        a1.pick_t2_activation(5);
        // Now their next draws should differ.
        assert_ne!(a1.decision_rng.next_u64(), a2.decision_rng.next_u64());
    }

    #[test]
    fn action_rng_count_increments_with_pick_action() {
        let mut a = RandomAgent::new(1);
        assert_eq!(a.action_rng_count, 0);
        // Directly invoke pick_action to confirm counter increments.
        let _ = a.pick_action(4);
        assert_eq!(a.action_rng_count, 1);
        let _ = a.pick_action(4);
        assert_eq!(a.action_rng_count, 2);
    }

    #[test]
    fn used_this_turn_starts_empty() {
        let a = RandomAgent::new(1);
        assert!(a.used_this_turn.is_empty());
        assert!(a.last_turn_key.is_none());
    }

    #[test]
    fn no_target_foul_deselects_and_activates_another_player() {
        // Regression (ogre seed 1, step 143): the turn-start eligible snapshot offered FOUL for a
        // player whose only adjacent prone/stunned victim had since moved away, so by activation the
        // foul had no legal target. Java's ParityRunner.sendFoulAction injects
        // ClientCommandActingPlayer(null,null,false) — a DESELECT that leaves the turn going — instead
        // of committing the foul. Rust used to commit Action::ActivatePlayer{Foul} with a null
        // defender, which StepInitSelecting's foul dispatch turned into an EndTurn (the half then
        // ended a full activation early vs Java). The agent must now deselect a no-target foul and
        // pick another player. Here "fouler" can ONLY foul (no legal victim) so it can never be
        // committed — the agent must fall through to "mover" for EVERY seed, and never EndTurn/Foul.
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState, PS_STANDING};
        use ffb_model::model::player::Player;
        fn mk(id: &str) -> Player {
            Player {
                id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
                player_type: PlayerType::Regular, gender: PlayerGender::Male,
                movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
                ..Default::default()
            }
        }
        for seed in 0u64..16 {
            let mut gs = new_game(seed);
            gs.game.home_playing = true;
            // Mid-turn fixture: the agent's turn<1 guard (Java ParityRunner `if (turn < 1)
            // EndTurn`, added for pass-block windows) must not fire here.
            gs.game.turn_data_home.turn_nr = 1;
            gs.game.turn_data_away.turn_nr = 1;
            gs.game.turn_mode = ffb_model::enums::TurnMode::Regular;
            gs.game.team_home.players.push(mk("fouler"));
            gs.game.team_home.players.push(mk("mover"));
            gs.game.team_away.players.push(mk("victim"));
            gs.game.field_model.set_player_coordinate("fouler", FieldCoordinate::new(10, 8));
            gs.game.field_model.set_player_state("fouler", PlayerState::new(PS_STANDING).change_active(true));
            gs.game.field_model.set_player_coordinate("mover", FieldCoordinate::new(5, 5));
            gs.game.field_model.set_player_state("mover", PlayerState::new(PS_STANDING).change_active(true));
            // victim is STANDING (not prone/stunned) → no legal foul target anywhere on the pitch.
            gs.game.field_model.set_player_coordinate("victim", FieldCoordinate::new(20, 1));
            gs.game.field_model.set_player_state("victim", PlayerState::new(PS_STANDING).change_active(true));

            let eligible = vec![
                ("fouler".to_string(), vec![PlayerAction::Foul]),
                ("mover".to_string(), vec![PlayerAction::Move]),
            ];
            gs.pending_prompt = Some(AgentPrompt::ActivatePlayer { eligible_players: eligible });

            let mut agent = RandomAgent::new_parity(seed);
            match agent.act(&gs) {
                Action::ActivatePlayer { player_id, player_action, .. } => {
                    assert_ne!(player_action, PlayerActionChoice::Foul,
                        "seed {seed}: a foul with no legal target must never be committed");
                    assert_eq!(player_id, "mover",
                        "seed {seed}: after deselecting the no-target foul the agent activates the mover");
                    assert_eq!(player_action, PlayerActionChoice::Move);
                }
                other => panic!("seed {seed}: expected the turn to continue with the mover, got {other:?}"),
            }
        }
    }
}

#[cfg(test)]
mod rng_trace_tests {
    use super::*;

    #[test]
    fn trace_seed1_actionrng_calls() {
        let seed = 1u64;
        let mut a = RandomAgent::new_parity(seed);
        
        // Pregame: consume 4 decision calls (coin, receive, kick_x, kick_y)
        for _ in 0..4 {
            let _ = a.decision_rng.next_u64();
        }
        
        // Decision call 5: player pick n=11
        let v = a.decision_rng.next_u64();
        eprintln!("decision[4] n=11: {} % 11 = {}", v, v as usize % 11);
        
        // Action call 1: action pick n=3 [Move,Block,Blitz]
        let v = a.action_rng.next_u64();
        eprintln!("action[0] n=3: {} % 3 = {}", v, v as usize % 3);
        
        // Action call 2: block target pick n=2 [home_01,home_03] 
        let v = a.action_rng.next_u64();
        eprintln!("action[1] n=2: {} % 2 = {}", v, v as usize % 2);
        
        // Action call 3: move target pick n=6 [(10,6),(10,7),(10,8),(11,6),(11,8),(12,8)]
        let v = a.action_rng.next_u64();
        eprintln!("action[2] n=6: {} % 6 = {}", v, v as usize % 6);
        
        // And n=7 (if targets list has 7 elements)
        let v2 = a.action_rng.next_u64(); // this is a different call
        eprintln!("(next) n=7: {} % 7 = {}", v2, v2 as usize % 7);
        
        assert!(true);
    }

    /// `ParityRunner.sendConcreteAction`'s switch handles only MOVE/STAND_UP/BLOCK/BLITZ*/FOUL*/
    /// PASS*/HAND_OVER*/THROW_TEAM_MATE*; every other PlayerAction hits `default:` and is DESELECTED
    /// with no state change. The Rust agent carried the action out instead — for the Goblin
    /// Bombardier's THROW_BOMB that meant Java burned a no-op activation while Rust threw the bomb
    /// and then ended the game outright (goblin bb2016 seed 1: `game_end` at i=3 vs Java's i=901).
    #[test]
    fn unhandled_acting_actions_mirror_the_parity_runner_deselect() {
        for pa in [
            PlayerActionChoice::Move,
            PlayerActionChoice::StandUp,
            PlayerActionChoice::Block,
            PlayerActionChoice::Blitz,
            PlayerActionChoice::StandUpBlitz,
            PlayerActionChoice::Foul,
            PlayerActionChoice::Pass,
            PlayerActionChoice::HandOff,
            PlayerActionChoice::ThrowTeamMate,
            // A kick declares through the same command as a throw and ParityRunner now routes
            // KICK_TEAM_MATE / KICK_TEAM_MATE_MOVE to sendThrowTeamMateAction. Until both sides did,
            // every kick was declared and instantly deselected, so Kick Team-Mate never executed in
            // ANY edition and the matrices were green because of it.
            PlayerActionChoice::KickTeamMate,
            PlayerActionChoice::ThrowBomb,
            // PUNT is forceDispatch and driven end-to-end since the dark_elf campaign
            // (ParityRunner isHandledActingAction gained PUNT/PUNT_MOVE).
            PlayerActionChoice::Punt,
        ] {
            assert!(is_handled_acting_action(pa), "{pa:?} has a sendConcreteAction arm");
        }
        for pa in [
            PlayerActionChoice::Stab,
            PlayerActionChoice::HypnoticGaze,
            PlayerActionChoice::Swoop,
            PlayerActionChoice::BreatheFire,
            PlayerActionChoice::ProjectileVomit,
            PlayerActionChoice::SecureTheBall,
        ] {
            assert!(!is_handled_acting_action(pa),
                "{pa:?} falls through to ParityRunner's default: arm and must deselect");
        }
    }
}
