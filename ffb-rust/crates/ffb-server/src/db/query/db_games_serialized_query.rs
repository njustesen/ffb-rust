/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbGamesSerializedQuery.
///
/// NOTE: Java's execute() gunzips the blob and deserializes it into a `GameState` via
/// `UtilJson.gunzip(...)` + `GameState.initFrom(...)`. There is no Rust GameState/gunzip-
/// JSON-to-GameState wiring available in this DB-layer crate yet, so this returns the raw
/// gzipped blob bytes exactly as read from the ResultSet; deserialization into GameState
/// is deferred to a later phase that wires up ffb-server's game-state layer.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "SELECT serialized FROM ffb_games_serialized WHERE id=?";

pub struct DbGamesSerializedQuery;

impl DbGamesSerializedQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_SERIALIZED_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Reads the gzipped JSON blob. Deserialization into a GameState is deferred (see
    /// module doc comment above).
    pub async fn execute(&self, conn: &mut Conn, game_state_id: i64) -> Result<Option<Vec<u8>>, DbError> {
        conn.exec_first(SQL, (game_state_id,)).await
    }
}

impl Default for DbGamesSerializedQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbGamesSerializedQuery::new();
        assert_eq!(q.get_id(), DbStatementId::GAMES_SERIALIZED_QUERY);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_serialized"));
    }

    #[test]
    fn sql_has_where_clause() {
        assert!(SQL.contains("WHERE id=?"));
    }
}
