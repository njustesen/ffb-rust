/// 1:1 translation of com.fumbbl.ffb.server.db.update.DbGamesInfoUpdate.
use crate::db::db_statement_id::DbStatementId;
use crate::db::update::db_games_info_update_parameter::DbGamesInfoUpdateParameter;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Value};

pub const SQL: &str = "UPDATE ffb_games_info SET \
    scheduled=?, started=?, finished=?, \
    coach_home=?, team_home_id=?, team_home_name=?, \
    coach_away=?, team_away_id=?, team_away_name=?, \
    half=?, turn=?, home_playing=?, status=?, testing=?, admin_mode=? \
    WHERE id=?";

pub struct DbGamesInfoUpdate;

impl DbGamesInfoUpdate {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_INFO_UPDATE
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it, the SQL is a
    /// plain constant and mysql_async prepares statements internally.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Mirrors DbGamesInfoUpdate.fillDbStatement(): binds every field of the
    /// update parameter in the exact column order of SQL, then executes it.
    pub async fn execute(
        &self,
        conn: &mut Conn,
        parameter: &DbGamesInfoUpdateParameter,
    ) -> Result<u64, DbError> {
        let params: Vec<Value> = vec![
            parameter.scheduled.into(),
            parameter.started.into(),
            parameter.finished.into(),
            parameter.coach_home.clone().into(),
            parameter.team_home_id.clone().into(),
            parameter.team_home_name.clone().into(),
            parameter.coach_away.clone().into(),
            parameter.team_away_id.clone().into(),
            parameter.team_away_name.clone().into(),
            parameter.half.into(),
            parameter.turn.into(),
            parameter.home_playing.into(),
            parameter.status.clone().into(),
            parameter.testing.into(),
            parameter.admin_mode.into(),
            parameter.id.into(),
        ];
        conn.exec_drop(SQL, params).await?;
        Ok(conn.affected_rows())
    }
}

impl Default for DbGamesInfoUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let s = DbGamesInfoUpdate::new();
        assert_eq!(s.get_id(), DbStatementId::GAMES_INFO_UPDATE);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_info"));
        assert!(SQL.to_uppercase().contains("UPDATE"));
    }

    #[test]
    fn sql_has_where_clause() {
        assert!(SQL.contains("WHERE"));
        assert!(SQL.contains("id=?"));
    }

    #[test]
    fn sql_has_set_columns() {
        assert!(SQL.contains("scheduled=?"));
        assert!(SQL.contains("started=?"));
        assert!(SQL.contains("finished=?"));
        assert!(SQL.contains("coach_home=?"));
        assert!(SQL.contains("team_home_id=?"));
        assert!(SQL.contains("team_home_name=?"));
        assert!(SQL.contains("coach_away=?"));
        assert!(SQL.contains("team_away_id=?"));
        assert!(SQL.contains("team_away_name=?"));
        assert!(SQL.contains("half=?"));
        assert!(SQL.contains("turn=?"));
        assert!(SQL.contains("home_playing=?"));
        assert!(SQL.contains("status=?"));
        assert!(SQL.contains("testing=?"));
        assert!(SQL.contains("admin_mode=?"));
    }
}
