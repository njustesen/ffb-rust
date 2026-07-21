/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DbTeamSetupsDelete.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "DELETE FROM ffb_team_setups WHERE (team_id = ? AND name = ?)";

pub struct DbTeamSetupsDelete;

impl DbTeamSetupsDelete {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::TEAM_SETUPS_DELETE
    }

    pub async fn execute(
        &self,
        conn: &mut Conn,
        team_id: &str,
        name: &str,
    ) -> Result<u64, DbError> {
        conn.exec_drop(SQL, (team_id, name)).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbTeamSetupsDelete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_is_delete() {
        assert!(SQL.trim_start().to_uppercase().starts_with("DELETE"));
    }
}
