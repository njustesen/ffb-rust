/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbUserSettingsQuery.
use crate::db::db_statement_id::DbStatementId;
use ffb_model::model::common_property::CommonProperty;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};
use std::collections::HashMap;

pub const SQL: &str = "SELECT * FROM ffb_user_settings WHERE coach=? ORDER BY name";

pub struct DbUserSettingsQuery {
    coach: String,
    settings: HashMap<String, String>,
}

impl DbUserSettingsQuery {
    pub fn new() -> Self {
        Self {
            coach: String::new(),
            settings: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::USER_SETTINGS_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// 1:1 translation of `execute(String pCoach)`, which delegates to
    /// `execute(pCoach, null)` in Java, storing results into `fSettings`.
    pub async fn execute(&mut self, conn: &mut Conn, coach: &str) -> Result<(), DbError> {
        self.coach = coach.to_string();
        self.settings.clear();
        // Java's QueryResult reads (coach, name, value) — coach is read and discarded.
        let rows: Vec<(String, String, String)> = conn.exec(SQL, (coach,)).await?;
        for (_coach, name, value) in rows {
            if CommonProperty::for_key(&name).is_some() {
                self.settings.insert(name, value);
            }
        }
        Ok(())
    }

    pub fn get_coach(&self) -> &str {
        &self.coach
    }

    pub fn get_setting_value(&self, setting_name: &str) -> Option<&str> {
        self.settings.get(setting_name).map(|s| s.as_str())
    }

    pub fn get_setting_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.settings.keys().map(|k| k.as_str()).collect();
        names.sort();
        names
    }

    pub fn get_setting_values(&self) -> Vec<&str> {
        self.settings.values().map(|v| v.as_str()).collect()
    }
}

impl Default for DbUserSettingsQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbUserSettingsQuery::new();
        assert_eq!(q.get_id(), DbStatementId::USER_SETTINGS_QUERY);
        assert_eq!(q.get_coach(), "");
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_user_settings"));
    }

    #[test]
    fn sql_has_where_and_order_by() {
        assert!(SQL.contains("WHERE coach=?"));
        assert!(SQL.contains("ORDER BY name"));
    }
}
