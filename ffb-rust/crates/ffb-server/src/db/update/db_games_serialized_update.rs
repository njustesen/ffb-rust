/// 1:1 translation of com.fumbbl.ffb.server.db.update.DbGamesSerializedUpdate.
use crate::db::db_statement_id::DbStatementId;
use crate::db::update::db_games_serialized_update_parameter::DbGamesSerializedUpdateParameter;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "UPDATE ffb_games_serialized SET serialized=? WHERE id=?";

pub struct DbGamesSerializedUpdate;

impl DbGamesSerializedUpdate {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_SERIALIZED_UPDATE
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Mirrors DbGamesSerializedUpdate.fillDbStatement(pFillBlob = true): binds the
    /// gzip-compressed serialized game blob followed by the id, then executes it.
    pub async fn execute(
        &self,
        conn: &mut Conn,
        parameter: &DbGamesSerializedUpdateParameter,
    ) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (parameter.data.clone(), parameter.id))
            .await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbGamesSerializedUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbGamesSerializedUpdate::new();
        assert_eq!(s.get_id(), DbStatementId::GAMES_SERIALIZED_UPDATE);
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

    #[test]
    fn sql_has_set_column() {
        assert!(SQL.contains("serialized=?"));
        assert!(SQL.to_uppercase().contains("UPDATE"));
    }
}
