/// Database utility functions for the FFB server — 1:1 translation of Java UtilServerDb.
///
/// Method bodies left as `todo!()` — DB wiring (SQLx) is Phase ZU work.
pub struct UtilServerDb;

impl UtilServerDb {
    pub fn load_game_state(game_id: i64) -> Option<String> {
        todo!("Phase ZU: DB implementation — load serialized game state by ID")
    }

    pub fn save_game_state(game_id: i64, serialized: &str) {
        todo!("Phase ZU: DB implementation — persist serialized game state")
    }

    pub fn delete_game_state(game_id: i64) {
        todo!("Phase ZU: DB implementation — remove game state from DB")
    }

    pub fn load_game_log(game_id: i64) -> Vec<String> {
        todo!("Phase ZU: DB implementation — load replay commands for game ID")
    }

    pub fn find_open_games_for_coach(coach: &str) -> Vec<i64> {
        todo!("Phase ZU: DB implementation — query open games for coach")
    }

    pub fn create_game_info(home_coach: &str, away_coach: &str) -> i64 {
        todo!("Phase ZU: DB implementation — insert games_info row and return generated ID")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util_server_db_exists() {
        // Verify the struct compiles — methods stub to todo!() for Phase ZU
        let _: fn(i64) = |_id| { /* Phase ZU */ };
    }

    #[test]
    fn test_module_accessible() {
        // Structural test: confirm UtilServerDb can be named from this module
        let _name = std::any::type_name::<UtilServerDb>();
        assert!(_name.contains("UtilServerDb"));
    }
}
