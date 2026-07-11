/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbGamesSerializedInsertParameter.
/// Extends DbGamesSerializedUpdateParameter; implements IDbUpdateWithGameState.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use super::db_games_serialized_insert::SQL;

pub struct DbGamesSerializedInsertParameter {
    id: i64,
    data: Vec<u8>,
    updated_rows: i32,
}

impl DbGamesSerializedInsertParameter {
    pub fn new(id: i64, data: Vec<u8>) -> Self {
        Self { id, data, updated_rows: 0 }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Async execute: runs the INSERT against the live connection.
    /// Replaces the JDBC execute_update() pattern from DefaultDbUpdateParameter.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.id, self.data.clone())).await?;
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
        let p = DbGamesSerializedInsertParameter::new(123, vec![1, 2, 3]);
        assert_eq!(p.get_id(), 123);
        assert_eq!(p.get_data(), &[1, 2, 3]);
    }

    #[test]
    fn initial_updated_rows() {
        let p = DbGamesSerializedInsertParameter::new(123, vec![]);
        assert_eq!(p.get_updated_rows(), 0);
    }
}
