/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbPlayerMarkersDeleteParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use crate::db::delete::db_player_markers_delete::SQL;

pub struct DbPlayerMarkersDeleteParameter {
    team_id: String,
    updated_rows: i32,
}

impl DbPlayerMarkersDeleteParameter {
    pub fn new(team_id: String) -> Self {
        Self { team_id, updated_rows: 0 }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the DELETE statement. Replaces JDBC execute_update() + PreparedStatement.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.team_id.as_str(),)).await?;
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
        let p = DbPlayerMarkersDeleteParameter::new("team1".to_string());
        assert_eq!(p.get_team_id(), "team1");
    }

    #[test]
    fn get_updated_rows_initial() {
        let p = DbPlayerMarkersDeleteParameter::new("team1".to_string());
        assert_eq!(p.get_updated_rows(), 0);
    }
}
