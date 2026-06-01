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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_players_on_pitch_is_11() {
        assert_eq!(MAX_PLAYERS_ON_PITCH, 11);
    }

    #[test]
    fn max_players_in_team_is_16() {
        assert_eq!(MAX_PLAYERS_IN_TEAM, 16);
    }

    #[test]
    fn turns_per_half_is_8() {
        assert_eq!(TURNS_PER_HALF, 8);
    }

    #[test]
    fn min_target_number_is_2() {
        assert_eq!(MIN_TARGET_NUMBER, 2);
    }
}
