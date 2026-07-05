//! Step-stack engine — 1:1 port of the Java `server/step` framework. Java is the sole ground
//! truth (the old monolithic engine has been removed).
//!
//! Spec: `docs/step_port/00_framework.md` (lifecycle), `INVARIANTS.md` (frozen primitives).
//! Concrete steps live in submodules and are dispatched via the `Step` enum (no `dyn`).
//! See `docs/step_port/20_steps/` for the per-step port entries.

#![allow(dead_code)] // scaffold: concrete steps land per 20_steps/ as Phase D progresses

pub(crate) mod framework;
pub mod driver;
pub mod bb2016;
pub mod bb2020;
pub mod bb2025;

// Infrastructure modules used by step implementations
pub mod util_server_re_roll;
pub mod util_server_injury;
pub mod util_server_steps;
pub mod util_server_catch_scatter_throw_in;
pub mod abstract_step_with_re_roll;
pub mod generator;
pub mod sequences;
pub mod action;

// Edition-specific modules
pub mod mixed;
pub mod phase;
pub mod game;
pub mod step_goto_label;
pub mod step_reset_to_move;

pub use framework::*;
pub use driver::*;
