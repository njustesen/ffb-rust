//! The step-engine agent boundary — a SEPARATE module from the driver, with a single clear
//! interface: `Agent::act(&GameState) -> Action`.
//!
//! Dependency direction is one-way: the agent reads the engine (`GameState`), never the
//! reverse. One `act` call per prompt — the agent inspects `gs.current_prompt()` (and `gs.game`
//! for legal-action queries) and returns the `Action` the driver should `apply`. State-in /
//! action-out: no separate response type.
//!
//! Two independent agent implementations live here:
//! - [`RandomAgent`] — the Java-parity driver. Its RNG-consumption order is load-bearing (see
//!   `AGENT_CONTRACT.md`); do not change its behavior.
//! - [`UniformAgent`] — a coverage-oriented driver with no RNG-stream discipline. It samples
//!   uniformly among all truly legal actions at every decision point, for "how much of the
//!   game's mechanic surface does random play exercise" runs.

mod random_agent;
mod uniform_agent;

pub use random_agent::RandomAgent;
pub use uniform_agent::UniformAgent;

use crate::action::Action;
use crate::step::GameState;

/// The step engine's decision-maker. Reads the game state (including the pending prompt) and
/// returns the action to apply. `&mut self` carries the agent's own RNG/turn state; `&GameState`
/// is read-only — the agent never mutates the engine.
pub trait Agent {
    fn act(&mut self, gs: &GameState) -> Action;
}
