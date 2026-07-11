/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbTeamSetupsDeleteParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use super::db_team_setups_delete::SQL;

pub struct DbTeamSetupsDeleteParameter {
    team_id: String,
    name: String,
    updated_rows: i32,
}

impl DbTeamSetupsDeleteParameter {
    pub fn new(team_id: String, name: String) -> Self {
        Self { team_id, name, updated_rows: 0 }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the DELETE statement for this parameter against the given connection.
    /// Corresponds to DbTeamSetupsDelete.execute() in the Java source.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (self.team_id.as_str(), self.name.as_str())).await?;
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
        let p = DbTeamSetupsDeleteParameter::new("team1".to_string(), "setup1".to_string());
        assert_eq!(p.get_team_id(), "team1");
        assert_eq!(p.get_name(), "setup1");
    }

    #[test]
    fn initial_updated_rows_is_zero() {
        let p = DbTeamSetupsDeleteParameter::new("team1".to_string(), "setup1".to_string());
        assert_eq!(p.get_updated_rows(), 0);
    }
}
