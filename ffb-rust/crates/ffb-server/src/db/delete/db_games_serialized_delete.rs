/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbGamesSerializedDelete.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "DELETE FROM ffb_games_serialized WHERE id=?";

pub struct DbGamesSerializedDelete;

impl DbGamesSerializedDelete {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_SERIALIZED_DELETE
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    pub async fn execute(&self, conn: &mut Conn, game_state_id: i64) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (game_state_id,)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbGamesSerializedDelete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbGamesSerializedDelete::new();
        assert_eq!(s.get_id(), DbStatementId::GAMES_SERIALIZED_DELETE);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_serialized"));
    }

    #[test]
    fn sql_has_where_clause() {
        assert!(SQL.contains("WHERE"));
        assert!(SQL.contains("id=?"));
    }
}
