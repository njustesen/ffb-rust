/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbUserSettingsInsert.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "INSERT INTO ffb_user_settings VALUES(?, ?, ?)";

pub struct DbUserSettingsInsert;

impl DbUserSettingsInsert {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::USER_SETTINGS_INSERT
    }

    pub async fn execute(
        &self,
        conn: &mut Conn,
        coach: &str,
        setting_name: &str,
        setting_value: &str,
    ) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (coach, setting_name, setting_value)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbUserSettingsInsert {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbUserSettingsInsert::new();
        assert_eq!(s.get_id(), DbStatementId::USER_SETTINGS_INSERT);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_user_settings"));
    }

    #[test]
    fn sql_has_three_placeholders() {
        assert_eq!(SQL.matches('?').count(), 3);
    }
}
