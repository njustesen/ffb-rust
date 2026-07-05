//! The step-engine agent boundary — a SEPARATE module from the driver, with a single clear
//! interface: `Agent::act(&GameState) -> Action`.
//!
//! Dependency direction is one-way: the agent reads the engine (`GameState`), never the
//! reverse. One `act` call per prompt — the agent inspects `gs.current_prompt()` (and `gs.game`
//! for legal-action queries) and returns the `Action` the driver should `apply`. State-in /
//! action-out: no separate response type.
//!
//! `RandomAgent` is the parity/coverage driver — it mirrors the Java `ParityRunner` decision/
//! action RNG contract (see `AGENT_CONTRACT.md` and `docs/step_port/INVARIANTS.md`). A single
//! shared instance drives BOTH sides (the runner plays both coaches); its two RNG streams are
//! kept distinct from the game dice by the seed XORs below.

use std::collections::HashSet;
use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::{PlayerAction, SkillId};

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{legal_block_targets, legal_foul_targets, legal_handoff_receivers, legal_pass_receivers, TeamSide};
use crate::step::GameState;

/// The step engine's decision-maker. Reads the game state (including the pending prompt) and
/// returns the action to apply. `&mut self` carries the agent's own RNG/turn state; `&GameState`
/// is read-only — the agent never mutates the engine.
pub trait Agent {
    fn act(&mut self, gs: &GameState) -> Action;
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
    skipped_this_turn: HashSet<String>,
    /// Turn key (half, turn_nr, home_playing) — detects when a new turn starts so we can
    /// clear `skipped_this_turn`.
    last_turn_key: Option<(i32, i32, bool)>,
    /// Debug: cumulative actionRng call count (for FFB_TRACE divergence diagnosis).
    action_rng_count: u64,
}

impl RandomAgent {
    /// Parity constructor: one shared agent for both sides, seeds matching Java byte-for-byte.
    pub fn new_parity(game_seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xDEAD_BEEF_CAFE_0001),
            action_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xC0FFEE_ACE0_0001),
            skipped_this_turn: HashSet::new(),
            last_turn_key: None,
            action_rng_count: 0,
        }
    }

    /// Coverage/visual constructor (no Java sync): both streams derive deterministically from
    /// `seed`. Callers use distinct seeds per side (e.g. `seed` / `seed ^ 0xFFFF_FFFF`).
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_ACE0_0001),
            skipped_this_turn: HashSet::new(),
            last_turn_key: None,
            action_rng_count: 0,
        }
    }

    /// Decision-RNG fair coin: `decisionRng.nextLong() % 2 == 0` (Java parity).
    fn pick_bool(&mut self) -> bool {
        self.decision_rng.next_u64() % 2 == 0
    }

    /// Decision-RNG uniform index in `[0, len)`: `remainderUnsigned(nextLong(), len)`.
    fn pick(&mut self, len: usize) -> usize {
        if len == 0 { 0 } else { (self.decision_rng.next_u64() as usize) % len }
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
                let x_raw = (self.decision_rng.next_u64() % 13) as i32;
                let y_raw = (self.decision_rng.next_u64() % 13) as i32;
                let x = if gs.game.home_playing { x_raw + 13 } else { x_raw };
                Action::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
            }
            // AGENT_CONTRACT.md §4-5: 1 decisionRng for player pick over remaining (§4 — EndTurn
            // is automatic when remaining is empty, NOT an explicit pick option),
            // 1 actionRng for action pick, 1 actionRng for block target when Block/Blitz.
            //
            // Java inactive-skip (ParityRunner tier>=3): players that are PRONE with active=false
            // (just recovered from STUNNED this turn) are in the eligible list but rejected when
            // picked. Each rejection consumes 1 decisionRng call. `skipped_this_turn` tracks
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
                    self.skipped_this_turn.clear();
                }

                // Build `remaining` as indices into eligible_players, excluding already-skipped.
                let mut remaining: Vec<usize> = (0..eligible_players.len())
                    .filter(|&i| !self.skipped_this_turn.contains(&eligible_players[i].0))
                    .collect();

                // Inactive-skip loop (mirrors Java ParityRunner while(true) pick loop).
                let (player_id, actions) = loop {
                    if remaining.is_empty() {
                        return Action::EndTurn;
                    }
                    let pick = self.pick(remaining.len()); // consumes 1 decisionRng
                    let player_list_idx = remaining.remove(pick);
                    let (pid, acts) = &eligible_players[player_list_idx];
                    // Check if the player is inactive (PRONE with active=false = just recovered
                    // from STUNNED this turn). Only PRONE+inactive players are skipped; STANDING
                    // players should always be active after refreshPlayersForTurnStart.
                    let ps = gs.game.field_model.player_state(pid);
                    let is_inactive = ps.map(|s| s.is_prone() && !s.is_active()).unwrap_or(false);
                    if is_inactive {
                        // Rejected: decisionRng already consumed; mark as skipped for this turn.
                        self.skipped_this_turn.insert(pid.clone());
                        continue;
                    }
                    break (pid, acts);
                };
                // Filter stale actions: mirrors Java ParityRunner.filterStaleActions — removes
                // Blitz/Block if blitz_used, Pass if pass_used, etc. The eligible
                // list was captured at turn start, so single-use actions may already be consumed.
                let td = if gs.game.home_playing { &gs.game.turn_data_home } else { &gs.game.turn_data_away };
                let live_actions: Vec<PlayerAction> = actions.iter().filter(|a| match a {
                    PlayerAction::Block | PlayerAction::Blitz => !td.blitz_used,
                    PlayerAction::Pass => !td.pass_used,
                    PlayerAction::HandOver => !td.hand_over_used,
                    PlayerAction::Foul => !td.foul_used,
                    _ => true,
                }).cloned().collect();
                let action_idx = self.pick_action(live_actions.len());
                let player_action = player_action_to_pac(&live_actions[action_idx]);
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_PICK pid={player_id} N={} idx={action_idx} action={player_action:?} arc={}", live_actions.len(), self.action_rng_count);
                }
                // For Block/Blitz: pick target from adjacent opponents
                // For Foul: pick foul target from adjacent prone/stunned opponents (1 actionRng call)
                let block_defender_id = match player_action {
                    PlayerActionChoice::Block
                    | PlayerActionChoice::Blitz => {
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
                    _ => None,
                };
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_ACT_END arc={}", self.action_rng_count);
                }
                Action::ActivatePlayer { player_id: player_id.clone(), player_action, block_defender_id }
            }
            // Move prompt: pick destination from legal squares using actionRng.
            Some(AgentPrompt::Move { player_id, squares }) => {
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_SMA pid={} N={}", player_id, squares.len());
                }
                if squares.is_empty() {
                    return Action::Move { path: vec![] };
                }
                let idx = self.pick_action(squares.len());
                if std::env::var("FFB_TRACE").is_ok() {
                    eprintln!("RUST_PICK pid={} N={} idx={} t=({},{})", player_id, squares.len(), idx, squares[idx].x, squares[idx].y);
                }
                Action::Move { path: vec![squares[idx]] }
            }
            // Pushback: uniformly sample from available squares (sorted by x,y for canonical
            // ordering that matches Java ParityRunner's sorted non-locked pushback list).
            // Consumes 1 decision_rng call — synced with Java ParityRunner PUSHBACK step case.
            Some(AgentPrompt::Pushback { squares, .. }) => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let mut sorted = squares.clone();
                sorted.sort_by_key(|c| (c.x, c.y));
                let idx = self.pick(sorted.len());
                Action::PushTo { coord: sorted[idx] }
            }
            // Follow-up: uniformly sample — consumes 1 decision_rng call.
            // Synced with Java ParityRunner FOLLOWUP_CHOICE dialog case.
            Some(AgentPrompt::FollowUp { .. }) => {
                Action::FollowUp { follow_up: self.pick_bool() }
            }
            // Block die selection: uniformly sample from available dice — 1 decision_rng call.
            // Synced with Java ParityRunner BLOCK_ROLL dialog case.
            Some(AgentPrompt::BlockChoice { dice, .. }) => {
                let idx = self.pick(dice.len().max(1));
                Action::BlockChoice { die_index: idx, target_id: None }
            }
            // Block choice with re-roll properties: consume 1 decision_rng call for consistency.
            // Synced with Java ParityRunner BLOCK_ROLL_PROPERTIES case.
            Some(AgentPrompt::BlockChoiceProperties { .. }) => {
                let _ = self.pick_bool();
                Action::BlockChoice { die_index: 0, target_id: None }
            }
            // Re-roll offer: uniformly sample use/decline — 1 decision_rng call.
            // Synced with Java ParityRunner RE_ROLL dialog case.
            Some(AgentPrompt::ReRollOffer { .. }) =>
                Action::UseReRoll { use_reroll: self.pick_bool() },
            // Skill use: uniformly sample use/decline — 1 decision_rng call.
            // Synced with Java ParityRunner SKILL_USE dialog case.
            // skill_id=Block is a placeholder (engine identifies the skill from step state, not
            // from the action's skill_id field when responding to a SkillUse prompt).
            Some(AgentPrompt::SkillUse { .. }) =>
                Action::UseSkill { skill_id: SkillId::Block, use_skill: self.pick_bool() },
            // Piling On: uniformly sample — 1 decision_rng call.
            // Synced with Java ParityRunner PILING_ON dialog case.
            Some(AgentPrompt::PilingOn { .. }) => {
                let use_it = self.pick_bool();
                Action::UseSkill { skill_id: SkillId::Block, use_skill: use_it }
            }
            // Apothecary choice: uniformly sample use/decline — 1 decision_rng call.
            // Synced with Java ParityRunner APOTHECARY_CHOICE dialog case.
            Some(AgentPrompt::ApothecaryChoice { player_id, .. }) =>
                Action::UseApothecary { player_id: player_id.clone(), use_apothecary: self.pick_bool() },
            Some(AgentPrompt::UseApothecary { .. }) =>
                Action::Acknowledge,
            // Interception: always decline — 0 RNG calls.
            // Java ParityRunner falls through to RandomStrategy which always sends sendInterceptorChoice(null,null).
            // Keeping both at 0 advances avoids RNG divergence.
            Some(AgentPrompt::Interception { .. }) =>
                Action::Intercept { attempt: false },
            // Touchback: pick uniformly from eligible players sorted by PlayerId — 1 decision_rng.
            // Synced with Java ParityRunner TOUCHBACK dialog case.
            Some(AgentPrompt::Touchback { eligible_players }) => {
                if eligible_players.is_empty() {
                    return Action::Acknowledge;
                }
                let mut sorted = eligible_players.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let idx = self.pick(sorted.len());
                Action::Touchback { player_id: sorted[idx].0.clone() }
            }
            // Argue the call: uniformly sample — 1 decision_rng call.
            // Synced with Java ParityRunner ARGUE_THE_CALL dialog case.
            Some(AgentPrompt::ArgueTheCall { .. }) =>
                Action::ArgueTheCall { argue: self.pick_bool() },
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
            // Inducement / pre-game: always decline / acknowledge with no RNG consumed.
            Some(AgentPrompt::BuyInducements { .. }) =>
                Action::BuyInducements { purchases: vec![] },
            Some(AgentPrompt::BuyPrayersAndInducements { .. }) =>
                Action::BuyInducements { purchases: vec![] },
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
fn player_action_to_pac(pa: &PlayerAction) -> PlayerActionChoice {
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
        // After KickBall the engine drives to the first ActivatePlayer prompt (0 extra dice).
        assert!(matches!(gs.current_prompt(), Some(AgentPrompt::ActivatePlayer { .. })),
            "engine waits at first ActivatePlayer after the kickoff");
        // The agent's decision RNG must not touch the game dice: the game dice are the pregame
        // (spectators: d6×4, weather: d6×2, coin: d2) plus the opening kickoff
        // (scatter: d8+d6, result roll: d6×2=7=BrilliantCoaching, coaching handler: d6×2) = 13.
        assert_eq!(gs.rng.call_count, 13, "agent decision RNG never perturbs the game-dice stream");
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
