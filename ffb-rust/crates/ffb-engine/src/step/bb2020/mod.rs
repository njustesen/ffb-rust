//! BB2020 step ports.
//!
//! ## Most of this module is NOT on the live path — read before trusting a file here
//!
//! `make_step_for(id, rules)` (`step/driver.rs`) has exactly ONE `Rules::Bb2020` arm,
//! `StepId::Prayer`. Every other StepId in a BB2020 game falls through to the SHARED (bb2025) step
//! set, so a BB2020 game does not run most of the files in this module. Of 104 step files here,
//! roughly 93 are never instantiated by any code path.
//!
//! The ~10 that DO run are reachable from the driver's edition-independent match arms — they are
//! the shared implementation for their StepId in EVERY edition, not just BB2020: `CloudBurster`,
//! `ReportStabInjury`, `StateMultipleRolls`, `AssignTouchdowns`, `BuyCardsAndInducements`,
//! `CheckStalling`, `SelectGazeTarget`, `SelectGazeTargetEnd`, `SetActingPlayerAndTeam`,
//! `SetActingTeam` (driver.rs:198-339), plus `StepPrayer` and the bb2020 `start_game` generator.
//!
//! Two consequences:
//!
//! 1. **Do not cite a file here as evidence of what BB2020 does at runtime.** It is a translation
//!    of the BB2020 Java class, but the engine is probably running the bb2025 twin instead. This
//!    has already produced one wrong rationale — see docs/PARITY_BB2020_CAMPAIGN.md, ITER117.
//! 2. **Fixing a BB2020 bug here usually changes nothing.** Edition-gate the shared bb2025 file
//!    instead; that is the campaign's standing pattern. Note the dead twin drifts silently, so
//!    keep it in sync by hand when you touch its live counterpart (ITER95 did this for all four
//!    `step_go_for_it.rs` files).
//!
//! These files are kept, not deleted: closing the structural gap means eventually routing
//! `make_step_for` AT them, the way BB2016 already routes Spectators, the kickoff chain,
//! MissedPass and the pass step-set. Deleting would discard a finished port.

pub mod block;
pub mod end;
pub mod foul;
pub mod gaze;
pub mod inducements;
pub mod kickoff;
pub mod move_;
pub mod multiblock;
pub mod pass;
pub mod shared;
pub mod special;
pub mod start;
pub mod ttm;

pub mod step_apothecary;
pub mod step_apply_kickoff_result;
pub mod step_baleful_hex;
pub mod step_black_ink;
pub mod step_blitz_turn;
pub mod step_breathe_fire;
pub mod step_catch_of_the_day;
pub mod step_end_furious_outburst;
pub mod step_end_turn;
pub mod step_handle_drop_player_context;
pub mod step_kickoff_scatter_roll;
pub mod step_look_into_my_eyes;
pub mod step_prayer;
pub mod step_prayers;
pub mod step_raiding_party;
pub mod step_select_blitz_target;
pub mod step_set_acting_player_and_team;
pub mod step_set_acting_team;
pub mod step_special_effect;
pub mod step_stalling_player;
pub mod step_state_multiple_rolls;
pub mod step_then_i_started_blastin;
pub mod step_treacherous;
pub mod step_wisdom_of_the_white_dwarf;

pub use step_apothecary::StepApothecary;
pub use step_apply_kickoff_result::StepApplyKickoffResult;
pub use step_baleful_hex::StepBalefulHex;
pub use step_black_ink::StepBlackInk;
pub use step_blitz_turn::StepBlitzTurn;
pub use step_breathe_fire::StepBreatheFire;
pub use step_catch_of_the_day::StepCatchOfTheDay;
pub use step_end_furious_outburst::StepEndFuriousOutburst;
pub use step_end_turn::StepEndTurn;
pub use step_handle_drop_player_context::StepHandleDropPlayerContext;
pub use step_kickoff_scatter_roll::StepKickoffScatterRoll;
pub use step_look_into_my_eyes::StepLookIntoMyEyes;
pub use step_prayer::StepPrayer;
pub use step_prayers::StepPrayers;
pub use step_raiding_party::StepRaidingParty;
pub use step_select_blitz_target::StepSelectBlitzTarget;
pub use step_set_acting_player_and_team::StepSetActingPlayerAndTeam;
pub use step_set_acting_team::StepSetActingTeam;
pub use step_special_effect::StepSpecialEffect;
pub use step_stalling_player::StepStallingPlayer;
pub use step_state_multiple_rolls::StepStateMultipleRolls;
pub use step_then_i_started_blastin::StepThenIStartedBlastin;
pub use step_treacherous::StepTreacherous;
pub use step_wisdom_of_the_white_dwarf::StepWisdomOfTheWhiteDwarf;
