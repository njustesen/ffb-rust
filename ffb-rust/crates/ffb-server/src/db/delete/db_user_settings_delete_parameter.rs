/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbUserSettingsDeleteParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use crate::db::delete::db_user_settings_delete::SQL;

pub struct DbUserSettingsDeleteParameter {
    coach: String,
    updated_rows: i32,
}

impl DbUserSettingsDeleteParameter {
    pub fn new(coach: String) -> Self {
        Self { coach, updated_rows: 0 }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the DELETE against the database.
    /// Translates Java: DbUserSettingsDelete.execute(IDbUpdateParameter)
    /// prepare() is a JDBC artifact — mysql_async uses the SQL constant directly.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.coach.clone(),)).await?;
        let affected = conn.affected_rows();
        self.updated_rows = affected as i32;
        Ok(affected)
    }

    // execute_update() was the JDBC-era entry point called through DbTransaction.
    // In mysql_async the async execute() above replaces it.
    // Kept as a doc comment to preserve the Java call-site record.
    // pub fn execute_update(&mut self) -> Result<(), String> { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbUserSettingsDeleteParameter::new("coach1".to_string());
        assert_eq!(p.get_coach(), "coach1");
    }

    #[test]
    fn get_updated_rows_initial() {
        let p = DbUserSettingsDeleteParameter::new("coach1".to_string());
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn sql_references_user_settings_table() {
        assert!(SQL.contains("ffb_user_settings"), "SQL must target ffb_user_settings table");
    }

    #[test]
    fn sql_is_delete_by_coach() {
        assert!(SQL.to_uppercase().starts_with("DELETE"), "SQL must be a DELETE statement");
        assert!(SQL.contains("coach"), "SQL must filter by coach column");
    }
}
