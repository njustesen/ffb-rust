/// Protocol version string (mirrors FantasyFootballConstants.VERSION from common.properties).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of players per team on the field.
pub const MAX_PLAYERS_ON_PITCH: usize = 11;

/// Maximum number of players in a team roster.
pub const MAX_PLAYERS_IN_TEAM: usize = 16;

/// Number of turns per half.
pub const TURNS_PER_HALF: i32 = 8;

/// Number of halves per game.
pub const HALVES_PER_GAME: i32 = 2;

/// Minimum d6 target number for most auto-pass rolls.
pub const MIN_TARGET_NUMBER: i32 = 2;

/// Maximum d6 target number (a 1 always fails).
pub const MAX_TARGET_NUMBER: i32 = 6;
