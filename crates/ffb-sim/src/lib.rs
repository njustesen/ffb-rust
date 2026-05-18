pub mod simulation;
pub mod setup;
pub mod evaluation;
pub mod canonical_strategy;
pub mod roster;
pub mod move_policy;
pub mod parity_log;

pub use canonical_strategy::CanonicalStrategy;
pub use roster::{make_team, make_star_player, star_player_def, PositionDef, StarPlayerDef};
