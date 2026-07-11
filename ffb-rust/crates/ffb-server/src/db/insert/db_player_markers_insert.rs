/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbPlayerMarkersInsert.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "INSERT INTO ffb_player_markers VALUES(?, ?, ?)";

pub struct DbPlayerMarkersInsert;

impl DbPlayerMarkersInsert {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::PLAYER_MARKERS_INSERT
    }

    pub async fn execute(
        &self,
        conn: &mut Conn,
        team_id: &str,
        player_id: &str,
        text: &str,
    ) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (team_id, player_id, text)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbPlayerMarkersInsert {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbPlayerMarkersInsert::new();
        assert_eq!(s.get_id(), DbStatementId::PLAYER_MARKERS_INSERT);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_player_markers"));
    }

    #[test]
    fn sql_has_three_placeholders() {
        assert_eq!(SQL.matches('?').count(), 3);
    }
}
