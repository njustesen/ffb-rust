/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbGamesInfoDeleteParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use crate::db::delete::db_games_info_delete::SQL;

pub struct DbGamesInfoDeleteParameter {
    game_state_id: i64,
    updated_rows: i32,
}

impl DbGamesInfoDeleteParameter {
    pub fn new(game_state_id: i64) -> Self {
        Self { game_state_id, updated_rows: 0 }
    }

    pub fn get_game_state_id(&self) -> i64 {
        self.game_state_id
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Execute the DELETE against an open connection.
    /// Replaces the JDBC prepare()/execute() pair — mysql_async handles statement
    /// preparation internally.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.game_state_id,)).await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbGamesInfoDeleteParameter::new(42);
        assert_eq!(p.get_game_state_id(), 42);
    }

    #[test]
    fn initial_updated_rows() {
        let p = DbGamesInfoDeleteParameter::new(42);
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn sql_targets_correct_table() {
        assert!(SQL.contains("ffb_games_info"));
        assert!(SQL.to_uppercase().contains("DELETE"));
    }
}
