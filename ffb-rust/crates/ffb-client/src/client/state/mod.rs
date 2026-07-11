//! 1:1 translation of `com.fumbbl.ffb.client.state` (client-side game phase state machines).
//!
//! All 3 root files plus the full `logic/` subtree (82 files across root/plugin/interaction/
//! bb2016/bb2020/bb2025/mixed) are now translated and wired — `client/state/` is complete
//! (Phase ZW.2 Batch D). See `TRANSLATION_TRACKER.md` for the per-file breakdown.

pub mod client_state;
pub mod client_state_factory;
pub mod i_player_popup_menu_keys;
pub mod logic;

pub use client_state::ClientState;
pub use client_state_factory::ClientStateFactory;
