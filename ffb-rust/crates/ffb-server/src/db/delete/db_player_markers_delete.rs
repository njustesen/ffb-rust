/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbPlayerMarkersDelete.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "DELETE FROM ffb_player_markers WHERE (team_id=?)";

pub struct DbPlayerMarkersDelete;

impl DbPlayerMarkersDelete {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::PLAYER_MARKERS_DELETE
    }

    pub async fn execute(&self, conn: &mut Conn, team_id: &str) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (team_id,)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbPlayerMarkersDelete {
    fn default() -> Self {
        Self::new()
    }
}

// Integration test skeleton — compiles but requires a live DB to run.
// Run manually with: cargo test -p ffb-server db_player_markers_delete -- --ignored
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbPlayerMarkersDelete::new();
        assert_eq!(s.get_id(), DbStatementId::PLAYER_MARKERS_DELETE);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_player_markers"));
    }

    #[test]
    fn sql_has_team_id_param() {
        assert!(SQL.contains("team_id=?"));
    }
}
