/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbPlayerMarkersQuery.
///
/// NOTE: Java's execute(GameState, boolean) resolves the team from the GameState and
/// then applies the returned (player_id, text) rows onto the FieldModel's PlayerMarkers.
/// There is no Rust GameState/FieldModel wiring available in this DB-layer crate yet, so
/// execute() takes the team_id directly (as the existing stub already did) and returns the
/// raw (player_id, text) rows for the caller to apply once that wiring exists.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "SELECT player_id, text FROM ffb_player_markers WHERE (team_id=?)";

pub struct DbPlayerMarkersQuery;

impl DbPlayerMarkersQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::PLAYER_MARKERS_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Returns (player_id, text) pairs for all markers in the given team.
    pub async fn execute(&self, conn: &mut Conn, team_id: &str) -> Result<Vec<(String, String)>, DbError> {
        if team_id.is_empty() {
            return Ok(Vec::new());
        }
        conn.exec_map(SQL, (team_id,), |(player_id, text): (String, String)| (player_id, text))
            .await
    }
}

impl Default for DbPlayerMarkersQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbPlayerMarkersQuery::new();
        assert_eq!(q.get_id(), DbStatementId::PLAYER_MARKERS_QUERY);
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
