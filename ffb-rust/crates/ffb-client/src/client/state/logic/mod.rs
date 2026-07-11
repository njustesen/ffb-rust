//! 1:1 translation of `com.fumbbl.ffb.client.state.logic` (pluggable client
//! logic modules). Only the modules translated so far are declared here; the
//! remaining PascalCase stub files (`*LogicModule.rs`, etc.) in this
//! directory are placeholders for future batches — see
//! `TRANSLATION_TRACKER.md`.

pub mod abstract_block_logic_module;
pub mod block_logic_extension;
pub mod client_action;
pub mod influences;
pub mod interaction;
pub mod logic_module;
pub mod move_logic_module;
pub mod plugin;
pub mod range_grid_state;
pub mod setup_logic_module;

pub use client_action::ClientAction;
pub use influences::Influences;
pub use logic_module::LogicModule;
pub use range_grid_state::RangeGridState;
