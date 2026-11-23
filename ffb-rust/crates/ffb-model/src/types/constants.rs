/// Java: `FantasyFootballConstants.VERSION` = `common.properties` `version=${project.version}`,
/// i.e. the ffb-java Maven project version (`ffb-java/ffb/pom.xml`). The Java client refuses a
/// server whose `serverVersion` is lower than its own, so this must track the Java pom, not Cargo.
pub const VERSION: &str = "3.2.0";

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
