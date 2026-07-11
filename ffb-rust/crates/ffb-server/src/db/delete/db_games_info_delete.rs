/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbGamesInfoDelete.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "DELETE FROM ffb_games_info WHERE id=?";

pub struct DbGamesInfoDelete;

impl DbGamesInfoDelete {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_INFO_DELETE
    }

    pub async fn execute(&self, conn: &mut Conn, game_state_id: i64) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (game_state_id,)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbGamesInfoDelete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbGamesInfoDelete::new();
        assert_eq!(s.get_id(), DbStatementId::GAMES_INFO_DELETE);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_info"));
    }

    #[test]
    fn sql_has_placeholder() {
        assert!(SQL.contains('?'));
    }
}
