//! 1:1 translation of `com.fumbbl.ffb.client.state.logic` (pluggable client
//! logic modules). Only the modules translated so far are declared here; the
//! remaining PascalCase stub files (`*LogicModule.rs`, etc.) in this
//! directory are placeholders for future batches — see
//! `TRANSLATION_TRACKER.md`.

pub mod client_action;
pub mod influences;
pub mod interaction;

pub use client_action::ClientAction;
pub use influences::Influences;
