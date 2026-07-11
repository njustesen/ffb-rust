//! 1:1 translation of `com.fumbbl.ffb.client.state.logic` (pluggable client
//! logic modules). Only the modules translated so far are declared here; the
//! remaining PascalCase stub files (`*LogicModule.rs`, etc.) in this
//! directory are placeholders for future batches — see
//! `TRANSLATION_TRACKER.md`.

pub mod abstract_block_logic_module;
pub mod bb2016;
pub mod bb2020;
pub mod bb2025;
pub mod blitz_logic_module;
pub mod block_logic_extension;
pub mod client_action;
pub mod dump_off_logic_module;
pub mod high_kick_logic_module;
pub mod illegal_substitution_logic_module;
pub mod influences;
pub mod interaction;
pub mod interception_logic_module;
pub mod kickoff_logic_module;
pub mod kickoff_return_logic_module;
pub mod logic_module;
pub mod login_logic_module;
pub mod mixed;
pub mod move_logic_module;
pub mod pass_block_logic_module;
pub mod place_ball_logic_module;
pub mod plugin;
pub mod pushback_logic_module;
pub mod quick_snap_logic_module;
pub mod range_grid_state;
pub mod replay_logic_module;
pub mod setup_logic_module;
pub mod solid_defence_logic_module;
pub mod spectate_logic_module;
pub mod start_game_logic_module;
pub mod swoop_logic_module;
pub mod throw_team_mate_logic_module;
pub mod touchback_logic_module;
pub mod wait_for_opponent_logic_module;
pub mod wait_for_setup_logic_module;
pub mod wizard_logic_module;

pub use client_action::ClientAction;
pub use influences::Influences;
pub use logic_module::LogicModule;
pub use range_grid_state::RangeGridState;
