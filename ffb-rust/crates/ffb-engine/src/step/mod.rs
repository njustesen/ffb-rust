//! Step-stack engine — 1:1 port of the Java `server/step` framework.
//!
//! Spec: `docs/step_port/00_framework.md` (lifecycle), `INVARIANTS.md` (frozen primitives).
//! This module is the new engine that replaces the monolithic `engine::GameEngine::apply`.
//! It is built alongside the monolith during the rewrite; the monolith is removed once this
//! can drive a full lineman game (Phase D).
//!
//! Concrete steps live in submodules and are dispatched via the `Step` enum (no `dyn`).
//! See `docs/step_port/20_steps/` for the per-step port entries.

#![allow(dead_code)] // scaffold: concrete steps land in Phase C/D

mod framework;
mod engine;
pub use framework::*;
pub use engine::*;
