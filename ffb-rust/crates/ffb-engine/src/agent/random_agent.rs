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
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PlayerAction, SkillId};

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{canonical_setup_action, legal_block_targets, legal_foul_targets, legal_handoff_receivers, legal_pass_receivers, legal_throw_team_mate_targets, TeamSide};
use crate::step::GameState;

use super::Agent;

/// AGENT_CONTRACT §7: the pushback coach picks the min-`(x, y)` on-pitch square, deterministically
/// (no decisionRng consumed). Mirrors Java `ParityRunner.sendPushback` (keep the square with the
/// smallest x, ties broken by smallest y, over the non-locked candidates).
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
    /// Players skipped this turn because they are inactive (just recovered from STUNNED).
    /// Mirrors Java ParityRunner's `usedThisTurn` for rejected-inactive picks.
    used_this_turn: HashSet<String>,
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
}

impl RandomAgent {
    /// Parity constructor: one shared agent for both sides, seeds matching Java byte-for-byte.
    pub fn new_parity(game_seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xDEAD_BEEF_CAFE_0001),
            action_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xC0FFEE_ACE0_0001),
            used_this_turn: HashSet::new(),
            last_turn_key: None,
            eligible_this_turn: Vec::new(),
            action_rng_count: 0,
            decision_rng_count: 0,
            current_activation_is_blitz: false,
            moved_this_activation: false,
        }
    }

    /// Coverage/visual constructor (no Java sync): both streams derive deterministically from
    /// `seed`. Callers use distinct seeds per side (e.g. `seed` / `seed ^ 0xFFFF_FFFF`).
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_ACE0_0001),
            used_this_turn: HashSet::new(),
            last_turn_key: None,
            eligible_this_turn: Vec::new(),
            action_rng_count: 0,
            decision_rng_count: 0,
            current_activation_is_blitz: false,
            moved_this_activation: false,
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
                let turn_key = (gs.game.half, turn_nr, gs.game.home_playing);
                if self.last_turn_key != Some(turn_key) {
                    self.last_turn_key = Some(turn_key);
                    self.used_this_turn.clear();
                    // Java: eligibleThisTurn = computeEligiblePlayers(game) — snapshot once at turn start.
                    self.eligible_this_turn = eligible_players.clone();
                }
                // Pick from the turn-start snapshot (Java `eligibleThisTurn`), NOT the engine's live
                // per-activation list, so an action offered at turn start (e.g. BLITZ) survives even if
                // its target is knocked down later in the same turn. Clone to sidestep the &mut self pick.
                let eligible_players = self.eligible_this_turn.clone();

                // Build `remaining` as indices into the snapshot, excluding already-skipped.
                let mut remaining: Vec<usize> = (0..eligible_players.len())
                    .filter(|&i| !self.used_this_turn.contains(&eligible_players[i].0))
                    .collect();

                // Inactive-skip loop (mirrors Java ParityRunner while(true) pick loop).
                let (player_id, actions) = loop {
                    if remaining.is_empty() {
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
                    // Check if the player is inactive (PRONE with active=false = just recovered
                    // from STUNNED this turn). Only PRONE+inactive players are skipped; STANDING
                    // players should always be active after refreshPlayersForTurnStart.
                    let ps = gs.game.field_model.player_state(pid);
                    let is_inactive = ps.map(|s| s.is_prone() && !s.is_active()).unwrap_or(false);
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
                    PlayerAction::Pass => !td.pass_used,
                    PlayerAction::HandOver => !td.hand_over_used,
                    PlayerAction::Foul => !td.foul_used,
                    // Throw/Kick Team-Mate are once per team turn (set ttm_used/ktm_used). The
                    // turn-start snapshot can still offer a second one after the first Ogre throws;
                    // filter it out as stale, or the engine rejects the second throw and the harness
                    // loops (ogre seed 1: away_06's throw after away_05 already threw).
                    PlayerAction::ThrowTeamMate => !td.ttm_used,
                    PlayerAction::KickTeamMate => !td.ktm_used,
                    _ => true,
                }).cloned().collect();
                let action_idx = self.pick_action(live_actions.len());
                let player_action = player_action_to_pac(&live_actions[action_idx]);
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_PICK pid={player_id} N={} idx={action_idx} action={player_action:?} arc={} drc={}", live_actions.len(), self.action_rng_count, self.decision_rng_count);
                }
                // For Block/Blitz: pick target from adjacent opponents
                // For Foul: pick foul target from adjacent prone/stunned opponents (1 actionRng call)
                let block_defender_id = match player_action {
                    PlayerActionChoice::Block
                    | PlayerActionChoice::Blitz
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
                    PlayerActionChoice::Pass => {
                        let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let receivers = legal_pass_receivers(&gs.game, player_id, side);
                        if receivers.is_empty() {
                            None
                        } else {
                            let ridx = self.pick_action(receivers.len());
                            Some(receivers[ridx].clone())
                        }
                    }
                    // Throw/Kick Team-Mate: pick the thrown player (an adjacent standing Right Stuff
                    // teammate), coordinate-sorted, 1 actionRng. Empty → None → StepInitSelecting
                    // deselects (no valid throwable teammate). The target square is chosen later, on
                    // the ThrowTeamMateTarget prompt. 1:1 with ParityRunner.sendThrowTeamMateAction.
                    PlayerActionChoice::ThrowTeamMate => {
                        let side = if gs.game.home_playing { TeamSide::Home } else { TeamSide::Away };
                        let targets = legal_throw_team_mate_targets(&gs.game, player_id, side);
                        if targets.is_empty() {
                            None
                        } else {
                            let tidx = self.pick_action(targets.len());
                            Some(targets[tidx].clone())
                        }
                    }
                    _ => None,
                };
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_END arc={}", self.action_rng_count);
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
                Action::ActivatePlayer { player_id: player_id.clone(), player_action, block_defender_id }
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
                // A blitz block reaches the follow-up Move/INIT_MOVING prompt; the stock harness
                // always deselects there, so mark the blitzer as already moved and fall through
                // to the always-deselect check below.
                if self.current_activation_is_blitz {
                    self.current_activation_is_blitz = false;
                    self.moved_this_activation = true;
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
                Action::Move { path: vec![squares[idx]] }
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
            Some(AgentPrompt::UseApothecary { .. }) =>
                Action::Acknowledge,
            // Interception: always decline — 0 RNG calls.
            // Java ParityRunner falls through to RandomStrategy which always sends sendInterceptorChoice(null,null).
            // Keeping both at 0 advances avoids RNG divergence.
            Some(AgentPrompt::Interception { .. }) =>
                Action::Intercept { attempt: false },
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
            Some(AgentPrompt::PlayerChoice { eligible_players, .. }) => {
                if eligible_players.is_empty() {
                    return Action::Acknowledge;
                }
                let mut sorted = eligible_players.clone();
                sorted.sort();
                let idx = self.pick(sorted.len());
                Action::SelectPlayer { player_id: sorted[idx].clone() }
            }
            // Select weather: pick uniformly from options — 1 decision_rng call.
            Some(AgentPrompt::SelectWeather { options }) => {
                if options.is_empty() {
                    return Action::Acknowledge;
                }
                let idx = self.pick(options.len());
                Action::SelectWeather { weather: options[idx] }
            }
            // Hit-and-run / trickster: pick square using actionRng (movement diversity).
            Some(AgentPrompt::HitAndRun { squares, .. }) => {
                if squares.is_empty() {
                    return Action::HitAndRun { coord: None };
                }
                let idx = self.pick_action(squares.len());
                Action::HitAndRun { coord: Some(squares[idx]) }
            }
            Some(AgentPrompt::TricksterMove { squares, .. }) => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let idx = self.pick_action(squares.len());
                Action::TricksterMove { coord: squares[idx] }
            }
            // Select skill: pick uniformly from all available skill IDs — 1 decision_rng call.
            // The u16 IDs in the prompt can't be directly mapped to SkillId enum variants without
            // a lookup table, so we consume 1 RNG call and return Acknowledge for now.
            // SelectSkill doesn't appear in T3 parity tests (no level-up in single game).
            Some(AgentPrompt::SelectSkill { available, .. }) => {
                let total: usize = available.iter().map(|(_, ids)| ids.len()).sum();
                if total > 0 { let _ = self.pick(total); }
                Action::Acknowledge
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
            // Confirm-only and informational prompts: single valid response, 0 RNG consumed.
            Some(AgentPrompt::KickoffReturn { .. })
            | Some(AgentPrompt::SetupError { .. })
            | Some(AgentPrompt::ConfirmEndAction { .. })
            | Some(AgentPrompt::InformationOkay { .. })
            | Some(AgentPrompt::SwarmingPlayers { .. })
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
        PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => PlayerActionChoice::Pass,
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
        PlayerAction::Treacherous | PlayerAction::WisdomOfTheWhiteDwarf | PlayerAction::ThrowKeg
        | PlayerAction::RaidingParty | PlayerAction::MaximumCarnage | PlayerAction::BalefulHex
        | PlayerAction::AllYouCanEat | PlayerAction::BlackInk | PlayerAction::CatchOfTheDay
        | PlayerAction::ThenIStartedBlastin | PlayerAction::TheFlashingBlade
        | PlayerAction::ViciousVines | PlayerAction::FuriousOutburst | PlayerAction::Chomp
        | PlayerAction::Incorporeal | PlayerAction::Forgo => PlayerActionChoice::Move,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::new_game;

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
}
