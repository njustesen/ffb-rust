/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbUserSettingsInsertParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use super::db_user_settings_insert::SQL;

pub struct DbUserSettingsInsertParameter {
    coach: String,
    setting_name: String,
    setting_value: String,
    updated_rows: i32,
}

impl DbUserSettingsInsertParameter {
    pub fn new(coach: String, setting_name: String, setting_value: String) -> Self {
        Self { coach, setting_name, setting_value, updated_rows: 0 }
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_setting_name(&self) -> &str {
        &self.setting_name
    }

    pub fn get_setting_value(&self) -> &str {
        &self.setting_value
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the INSERT statement. Replaces JDBC execute_update() + PreparedStatement.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(
            SQL,
            (self.coach.as_str(), self.setting_name.as_str(), self.setting_value.as_str()),
        )
        .await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbUserSettingsInsertParameter::new(
            "coach1".to_string(),
            "setting_a".to_string(),
            "value_b".to_string(),
        );
        assert_eq!(p.get_coach(), "coach1");
        assert_eq!(p.get_setting_name(), "setting_a");
        assert_eq!(p.get_setting_value(), "value_b");
    }

    #[test]
    fn get_updated_rows_initial() {
        let p = DbUserSettingsInsertParameter::new(
            "coach1".to_string(),
            "setting_a".to_string(),
            "value_b".to_string(),
        );
        assert_eq!(p.get_updated_rows(), 0);
    }
}
