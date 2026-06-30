/// BB2025-specific step and command implementations.
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025`.
pub mod block;
pub mod command;
pub mod end;
pub mod foul;
pub mod inducements;
pub mod kickoff;
pub mod move_;
pub mod mutliblock;
pub mod pass;
pub mod punt;
pub mod shared;
pub mod special;
pub mod start;
pub mod ttm;

// Root bb2025 steps (Java: step/bb2025/StepFoo.java)
pub mod step_auto_gaze_zoat;
pub mod step_baleful_hex;
pub mod step_black_ink;
pub mod step_catch_of_the_day;
pub mod step_end_furious_outburst;
pub mod step_end_turn;
pub mod step_look_into_my_eyes;
pub mod step_prayer;
pub mod step_raiding_party;
pub mod step_select_blitz_target;
pub mod step_then_i_started_blastin;
pub mod step_treacherous;
pub mod step_wisdom_of_the_white_dwarf;
