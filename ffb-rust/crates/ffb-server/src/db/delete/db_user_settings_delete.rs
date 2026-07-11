/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbUserSettingsDelete.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "DELETE FROM ffb_user_settings WHERE coach=?";

pub struct DbUserSettingsDelete;

impl DbUserSettingsDelete {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::USER_SETTINGS_DELETE
    }

    pub async fn execute(&self, conn: &mut Conn, coach: &str) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (coach,)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbUserSettingsDelete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbUserSettingsDelete::new();
        assert_eq!(s.get_id(), DbStatementId::USER_SETTINGS_DELETE);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_user_settings"));
    }

    #[test]
    fn sql_has_coach_param() {
        assert!(SQL.contains("coach=?"));
    }


}
