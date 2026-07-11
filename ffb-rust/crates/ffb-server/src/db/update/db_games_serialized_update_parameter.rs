/// 1:1 translation of com.fumbbl.ffb.server.db.update.DbGamesSerializedUpdateParameter.
/// Implements IDbUpdateWithGameState. Java holds the GameState directly and gzips
/// its JSON representation lazily inside gzip(); the GameState -> JSON pipeline is
/// not yet wired up in ffb-server, so this parameter instead holds the already
/// gzip-compressed serialized game bytes (set via `new`/`set_data`) — the id +
/// serialized blob bytes noted for this table.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use super::db_games_serialized_update::SQL;

pub struct DbGamesSerializedUpdateParameter {
    pub id: i64,
    /// Gzip-compressed serialized game bytes bound to the `serialized` column.
    pub data: Vec<u8>,
    /// JSON representation length (before gzip), mirrors Java's length().
    pub json_length: usize,
    updated_rows: i32,
}

impl DbGamesSerializedUpdateParameter {
    pub fn new(id: i64) -> Self {
        Self { id, data: Vec::new(), json_length: 0, updated_rows: 0 }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn length(&self) -> usize {
        self.json_length
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Returns the gzipped bytes for the serialized game state.
    /// Mirrors Java's gzip(), which compresses the GameState's JSON on demand;
    /// here the bytes are already stored on the struct (see module doc comment).
    pub fn gzip(&self) -> Result<Vec<u8>, String> {
        Ok(self.data.clone())
    }

    /// Async execute: runs the UPDATE against the live connection.
    /// Replaces the JDBC execute_update() pattern from DefaultDbUpdateParameter.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.data.clone(), self.id)).await?;
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
        let p = DbGamesSerializedUpdateParameter::new(1);
        assert_eq!(p.get_id(), 1);
        assert_eq!(p.length(), 0);
        assert_eq!(p.get_updated_rows(), 0);
        assert!(p.data.is_empty());
    }

    #[test]
    fn gzip_returns_stored_bytes() {
        let mut p = DbGamesSerializedUpdateParameter::new(2);
        p.data = vec![1, 2, 3];
        assert_eq!(p.gzip().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn sql_targets_correct_table() {
        assert!(SQL.contains("ffb_games_serialized"));
        assert!(SQL.to_uppercase().contains("UPDATE"));
    }
}
