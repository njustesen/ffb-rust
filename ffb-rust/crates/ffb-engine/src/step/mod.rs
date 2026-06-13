//! Step-stack engine — 1:1 port of the Java `server/step` framework. Java is the sole ground
//! truth (the old monolithic engine has been removed).
//!
//! Spec: `docs/step_port/00_framework.md` (lifecycle), `INVARIANTS.md` (frozen primitives).
//! Concrete steps live in submodules and are dispatched via the `Step` enum (no `dyn`).
//! See `docs/step_port/20_steps/` for the per-step port entries.

#![allow(dead_code)] // scaffold: concrete steps land per 20_steps/ as Phase D progresses

mod framework;
mod engine;
pub use framework::*;
pub use engine::*;
