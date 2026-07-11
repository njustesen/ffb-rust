/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbTestGameListQuery.
use crate::admin::admin_list_entry::AdminListEntry;
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Row, Value};

pub const SQL: &str = "SELECT id, started, finished, last_updated, coach_home, team_home_id, \
    team_home_name, coach_away, team_away_id, team_away_name, half, turn, status, testing \
    FROM ffb_games_info \
    WHERE testing=true \
    ORDER BY started ASC \
    LIMIT ?";

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

pub struct DbTestGameListQuery;

impl DbTestGameListQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::TEST_GAME_LIST_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Mirrors Java's `if (limit < 1) return;` early-out.
    pub async fn execute(&self, conn: &mut Conn, limit: i64) -> Result<Vec<AdminListEntry>, DbError> {
        if limit < 1 {
            return Ok(Vec::new());
        }
        conn.exec_map(SQL, (limit,), |mut row: Row| {
            let mut col = 0usize;
            let game_id: i64 = row.take(col).unwrap_or_default();
            col += 1;
            let started: Option<i64> = row.take::<Value, _>(col).and_then(value_to_millis);
            col += 1;
            let finished: Option<i64> = row.take::<Value, _>(col).and_then(value_to_millis);
            col += 1;
            let last_updated: Option<i64> = row.take::<Value, _>(col).and_then(value_to_millis);
            col += 1;
            let team_home_coach: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_home_id: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_home_name: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_coach: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_id: String = row.take(col).unwrap_or_default();
            col += 1;
            let team_away_name: String = row.take(col).unwrap_or_default();
            col += 1;
            let half: i32 = row.take(col).unwrap_or_default();
            col += 1;
            let turn: i32 = row.take(col).unwrap_or_default();
            col += 1;
            let status: String = row.take(col).unwrap_or_default();
            col += 1;
            let test_mode: bool = row.take(col).unwrap_or_default();

            let mut entry = AdminListEntry::new(game_id);
            entry.started = started;
            entry.finished = finished;
            entry.last_updated = last_updated;
            entry.team_home_coach = team_home_coach;
            entry.team_home_id = team_home_id;
            entry.team_home_name = team_home_name;
            entry.team_away_coach = team_away_coach;
            entry.team_away_id = team_away_id;
            entry.team_away_name = team_away_name;
            entry.half = half;
            entry.turn = turn;
            entry.status = status;
            entry.test_mode = test_mode;
            entry
        })
        .await
    }
}

impl Default for DbTestGameListQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbTestGameListQuery::new();
        assert_eq!(q.get_id(), DbStatementId::TEST_GAME_LIST_QUERY);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("testing=true"));
    }

    #[test]
    fn sql_has_limit_placeholder() {
        assert!(SQL.trim_end().ends_with("LIMIT ?"));
    }
}
