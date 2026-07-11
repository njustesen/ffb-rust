/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbTeamSetupsInsert.
/// 35 placeholders: team_id, name, then 11 × (player_nr, x, y).
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Value};

pub const SQL: &str = "INSERT INTO ffb_team_setups VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

pub struct DbTeamSetupsInsert;

impl DbTeamSetupsInsert {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::TEAM_SETUPS_INSERT
    }

    /// Java: fillDbStatement() — binds team_id, name, then loops over the (at most 11)
    /// player_numbers/x/y entries, setting NULL for any of the 11 slots beyond the
    /// supplied array length.
    pub async fn execute(
        &self,
        conn: &mut Conn,
        team_id: &str,
        name: &str,
        player_numbers: &[u8],
        x_coordinates: &[u8],
        y_coordinates: &[u8],
    ) -> Result<u64, DbError> {
        let mut params: Vec<Value> = Vec::with_capacity(35);
        params.push(Value::from(team_id));
        params.push(Value::from(name));
        let n = player_numbers.len().min(11);
        for i in 0..11usize {
            if i < n {
                params.push(Value::from(player_numbers[i]));
                params.push(Value::from(x_coordinates[i]));
                params.push(Value::from(y_coordinates[i]));
            } else {
                params.push(Value::NULL);
                params.push(Value::NULL);
                params.push(Value::NULL);
            }
        }
        conn.exec_drop(SQL, params).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbTeamSetupsInsert {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbTeamSetupsInsert::new();
        assert_eq!(s.get_id(), DbStatementId::TEAM_SETUPS_INSERT);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_team_setups"));
    }

    #[test]
    fn sql_has_35_placeholders() {
        assert_eq!(SQL.matches('?').count(), 35);
    }
}
