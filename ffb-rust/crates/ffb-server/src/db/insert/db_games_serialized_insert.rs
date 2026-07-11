/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbGamesSerializedInsert.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "INSERT INTO ffb_games_serialized VALUES(?,?)";

pub struct DbGamesSerializedInsert;

impl DbGamesSerializedInsert {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_SERIALIZED_INSERT
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    pub async fn execute(&self, conn: &mut Conn, id: i64, data: &[u8]) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (id, data)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbGamesSerializedInsert {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbGamesSerializedInsert::new();
        assert_eq!(s.get_id(), DbStatementId::GAMES_SERIALIZED_INSERT);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_serialized"));
    }

    #[test]
    fn sql_has_two_placeholders() {
        assert_eq!(SQL.matches('?').count(), 2);
    }

    #[test]
    fn sql_is_insert() {
        assert!(SQL.trim_start().to_uppercase().starts_with("INSERT"));
    }
}
