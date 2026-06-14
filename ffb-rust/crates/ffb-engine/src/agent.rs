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

use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_model::prompts::AgentPrompt;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;

use crate::action::{Action, PlayerActionChoice};
use crate::legal_actions::{legal_block_targets, TeamSide};
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
}

impl RandomAgent {
    /// Parity constructor: one shared agent for both sides, seeds matching Java byte-for-byte.
    pub fn new_parity(game_seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xDEAD_BEEF_CAFE_0001),
            action_rng: Xoshiro256StarStar::seed_from_u64(game_seed ^ 0xC0FFEE_ACE0_0001),
        }
    }

    /// Coverage/visual constructor (no Java sync): both streams derive deterministically from
    /// `seed`. Callers use distinct seeds per side (e.g. `seed` / `seed ^ 0xFFFF_FFFF`).
    pub fn new(seed: u64) -> Self {
        RandomAgent {
            decision_rng: Xoshiro256StarStar::seed_from_u64(seed),
            action_rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_ACE0_0001),
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
            Some(AgentPrompt::ActivatePlayer { eligible_players }) => {
                let n = eligible_players.len();
                if n == 0 {
                    return Action::EndTurn;
                }
                let player_idx = self.pick(n); // pick from [0, n) — no EndTurn slot
                let (player_id, actions) = &eligible_players[player_idx];
                let action_idx = self.pick_action(actions.len());
                let player_action = player_action_to_pac(&actions[action_idx]);
                // For Block/Blitz: pick target from adjacent opponents
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
                            Some(targets[tidx].clone())
                        }
                    }
                    _ => None,
                };
                Action::ActivatePlayer { player_id: player_id.clone(), player_action, block_defender_id }
            }
            // Move prompt: pick destination from legal squares using actionRng.
            Some(AgentPrompt::Move { squares, .. }) => {
                if squares.is_empty() {
                    return Action::Move { path: vec![] };
                }
                let idx = self.pick_action(squares.len());
                Action::Move { path: vec![squares[idx]] }
            }
            // Stubs for prompts that the engine currently handles internally (auto-push/follow-up)
            // but may emit in future phases. Handled here to prevent panics during T3 games.
            Some(AgentPrompt::Pushback { squares, .. }) => {
                if squares.is_empty() {
                    return Action::Acknowledge;
                }
                let idx = self.pick_action(squares.len());
                Action::PushTo { coord: squares[idx] }
            }
            Some(AgentPrompt::FollowUp { .. }) => {
                Action::FollowUp { follow_up: true }
            }
            Some(AgentPrompt::BlockChoice { dice, .. }) => {
                if dice.is_empty() {
                    return Action::BlockChoice { die_index: 0 };
                }
                let idx = self.pick_action(dice.len());
                Action::BlockChoice { die_index: idx }
            }
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
        PlayerAction::Move       => PlayerActionChoice::Move,
        PlayerAction::Block      => PlayerActionChoice::Block,
        PlayerAction::Blitz      => PlayerActionChoice::Blitz,
        PlayerAction::StandUp    => PlayerActionChoice::StandUp,
        PlayerAction::StandUpBlitz => PlayerActionChoice::StandUpBlitz,
        PlayerAction::Foul       => PlayerActionChoice::Foul,
        PlayerAction::Pass       => PlayerActionChoice::Pass,
        PlayerAction::HandOver   => PlayerActionChoice::HandOff,
        other => unimplemented!("player_action_to_pac: unhandled {other:?}"),
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
        // (d3,d3,d6,d6 + coin d2) plus the opening kickoff (scatter d8,d6 + result d6,d6 +
        // Cheering Fans d6,d6 + bounce d8) = 12.
        assert_eq!(gs.rng.call_count, 12, "agent decision RNG never perturbs the game-dice stream");
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
