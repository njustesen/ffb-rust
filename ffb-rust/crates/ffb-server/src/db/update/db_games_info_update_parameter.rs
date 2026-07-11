/// 1:1 translation of com.fumbbl.ffb.server.db.update.DbGamesInfoUpdateParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Value};

use super::db_games_info_update::SQL;

pub struct DbGamesInfoUpdateParameter {
    pub id: i64,
    pub scheduled: Option<i64>,
    pub started: Option<i64>,
    pub finished: Option<i64>,
    pub coach_home: String,
    pub team_home_id: String,
    pub team_home_name: String,
    pub coach_away: String,
    pub team_away_id: String,
    pub team_away_name: String,
    pub half: i8,
    pub turn: i8,
    pub home_playing: bool,
    pub status: String,
    pub testing: bool,
    pub admin_mode: bool,
    updated_rows: i32,
}

impl DbGamesInfoUpdateParameter {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            scheduled: None,
            started: None,
            finished: None,
            coach_home: String::new(),
            team_home_id: String::new(),
            team_home_name: String::new(),
            coach_away: String::new(),
            team_away_id: String::new(),
            team_away_name: String::new(),
            half: 0,
            turn: 0,
            home_playing: false,
            status: String::new(),
            testing: false,
            admin_mode: false,
            updated_rows: 0,
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Async execute: runs the UPDATE against the live connection.
    /// Replaces the JDBC execute_update() pattern from DefaultDbUpdateParameter.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        let params: Vec<Value> = vec![
            self.scheduled.into(),
            self.started.into(),
            self.finished.into(),
            self.coach_home.clone().into(),
            self.team_home_id.clone().into(),
            self.team_home_name.clone().into(),
            self.coach_away.clone().into(),
            self.team_away_id.clone().into(),
            self.team_away_name.clone().into(),
            self.half.into(),
            self.turn.into(),
            self.home_playing.into(),
            self.status.clone().into(),
            self.testing.into(),
            self.admin_mode.into(),
            self.id.into(),
        ];
        conn.exec_drop(SQL, params).await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }

    // execute_update() was the JDBC path via DbTransaction — not applicable in mysql_async.
    // Use execute(&mut conn) instead.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbGamesInfoUpdateParameter::new(99);
        assert_eq!(p.get_id(), 99);
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn default_fields() {
        let p = DbGamesInfoUpdateParameter::new(1);
        assert_eq!(p.scheduled, None);
        assert_eq!(p.started, None);
        assert_eq!(p.finished, None);
        assert_eq!(p.coach_home, "");
        assert_eq!(p.half, 0);
        assert_eq!(p.turn, 0);
        assert!(!p.home_playing);
        assert!(!p.testing);
        assert!(!p.admin_mode);
    }

    #[test]
    fn sql_targets_correct_table() {
        assert!(SQL.contains("ffb_games_info"));
        assert!(SQL.to_uppercase().contains("UPDATE"));
    }
}
