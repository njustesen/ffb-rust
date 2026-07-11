/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbGameListQueryOpenGamesByCoach.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Row, Value};

pub const SQL: &str = "SELECT id, started, team_home_id, team_home_name, coach_home, \
    team_away_id, team_away_name, coach_away \
    FROM ffb_games_info \
    WHERE finished IS NULL AND (coach_home=? OR coach_away=?)";

// NOTE: ffb-model's GameListEntry (crate::model::game_list_entry) does not carry the
// per-team id/name/coach columns that Java's GameListEntry does, so a dedicated row
// struct mirrors the Java ResultSet column reads 1:1 until that model is widened.
/// 1:1 translation of the row shape built by DbGameListQueryOpenGamesByCoach.execute().
#[derive(Debug, Clone, PartialEq)]
pub struct OpenGameByCoachRow {
    pub game_id: i64,
    pub started: Option<i64>,
    pub team_home_id: String,
    pub team_home_name: String,
    pub team_home_coach: String,
    pub team_away_id: String,
    pub team_away_name: String,
    pub team_away_coach: String,
}

/// Manual MySQL DATETIME/TIMESTAMP -> epoch-millis conversion (mysql_common has no
/// chrono/time feature enabled in this workspace).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn value_to_millis(v: Value) -> Option<i64> {
    match v {
        Value::Date(y, mo, d, h, mi, s, micro) => {
            if y == 0 && mo == 0 && d == 0 {
                return None;
            }
            let days = days_from_civil(y as i64, mo as u32, d as u32);
            Some(
                days * 86_400_000
                    + (h as i64) * 3_600_000
                    + (mi as i64) * 60_000
                    + (s as i64) * 1_000
                    + (micro as i64) / 1000,
            )
        }
        _ => None,
    }
}

pub struct DbGameListQueryOpenGamesByCoach;

impl DbGameListQueryOpenGamesByCoach {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAME_LIST_QUERY_OPEN_GAMES_BY_COACH
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Returns nothing (mirrors Java's early-return) if `coach` is empty.
    pub async fn execute(&self, conn: &mut Conn, coach: &str) -> Result<Vec<OpenGameByCoachRow>, DbError> {
        if coach.is_empty() {
            return Ok(Vec::new());
        }
        conn.exec_map(SQL, (coach, coach), |mut row: Row| {
            let mut col = 0usize;
            let game_id: i64 = row.take(col).unwrap_or_default();
            col += 1;
            let started: Option<i64> = row.take::<Value, _>(col).and_then(value_to_millis);
            col += 1;
            let team_home_id: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_home_name: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_home_coach: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_id: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_name: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_coach: String = row.take(col).unwrap_or_default();

            OpenGameByCoachRow {
                game_id,
                started,
                team_home_id,
                team_home_name,
                team_home_coach,
                team_away_id,
                team_away_name,
                team_away_coach,
            }
        })
        .await
    }
}

impl Default for DbGameListQueryOpenGamesByCoach {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbGameListQueryOpenGamesByCoach::new();
        assert_eq!(q.get_id(), DbStatementId::GAME_LIST_QUERY_OPEN_GAMES_BY_COACH);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("coach_home=?"));
    }
}
