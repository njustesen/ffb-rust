/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbPlayerMarkersInsertParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use crate::db::i_db_update_parameter::IDbUpdateParameter;

use super::db_player_markers_insert::SQL;

#[derive(Clone)]
pub struct DbPlayerMarkersInsertParameter {
    team_id: String,
    player_id: String,
    text: String,
    updated_rows: i32,
}

impl DbPlayerMarkersInsertParameter {
    pub fn new(team_id: String, player_id: String, text: String) -> Self {
        Self { team_id, player_id, text, updated_rows: 0 }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_player_id(&self) -> &str {
        &self.player_id
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the INSERT statement. Replaces JDBC execute_update() + PreparedStatement.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(
            SQL,
            (self.team_id.as_str(), self.player_id.as_str(), self.text.as_str()),
        )
        .await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }
}

impl IDbUpdateParameter for DbPlayerMarkersInsertParameter {
    fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// The synchronous execute_update() path is a JDBC artifact and is not used
    /// in the async mysql_async port; use the async execute(&mut self, conn) instead.
    fn execute_update(&mut self) -> Result<(), String> {
        Err("execute_update() is a JDBC artifact; use async execute() instead".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbPlayerMarkersInsertParameter::new(
            "t1".to_string(),
            "p1".to_string(),
            "text".to_string(),
        );
        assert_eq!(p.get_team_id(), "t1");
        assert_eq!(p.get_player_id(), "p1");
        assert_eq!(p.get_text(), "text");
    }

    #[test]
    fn get_updated_rows_initial() {
        let p = DbPlayerMarkersInsertParameter::new(
            "t1".to_string(),
            "p1".to_string(),
            "text".to_string(),
        );
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn execute_update_is_jdbc_artifact() {
        let mut p = DbPlayerMarkersInsertParameter::new(
            "t1".to_string(),
            "p1".to_string(),
            "text".to_string(),
        );
        assert!(IDbUpdateParameter::execute_update(&mut p).is_err());
    }
}
