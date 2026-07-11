/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbGamesSerializedDeleteParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use super::db_games_serialized_delete::SQL;

pub struct DbGamesSerializedDeleteParameter {
    game_state_id: i64,
    updated_rows: i32,
}

impl DbGamesSerializedDeleteParameter {
    pub fn new(game_state_id: i64) -> Self {
        Self { game_state_id, updated_rows: 0 }
    }

    pub fn get_game_state_id(&self) -> i64 {
        self.game_state_id
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Async execute: runs the DELETE against the live connection.
    /// Replaces the JDBC execute_update() pattern from DefaultDbUpdateParameter.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.game_state_id,)).await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }

    // execute_update() was the JDBC path via DbTransaction — not applicable in mysql_async.
    // Use execute(&mut conn) instead.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbGamesSerializedDeleteParameter::new(7);
        assert_eq!(p.get_game_state_id(), 7);
    }

    #[test]
    fn initial_updated_rows() {
        let p = DbGamesSerializedDeleteParameter::new(7);
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn sql_targets_correct_table() {
        assert!(SQL.contains("ffb_games_serialized"));
        assert!(SQL.to_uppercase().contains("DELETE"));
    }
}
