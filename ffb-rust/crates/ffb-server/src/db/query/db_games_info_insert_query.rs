/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbGamesInfoInsertQuery.
/// Uses RETURN_GENERATED_KEYS to read back the auto-incremented game id.
///
/// NOTE: Java's execute() takes a full GameState/Game object and pulls scheduled/started/
/// finished/coach/team fields off of it before setting the generated id back onto
/// `pGameState.getGame()`. There is no Rust GameState/Game wiring available to this
/// DB-layer crate yet (see db_games_serialized_query.rs), so execute() instead takes the
/// individual column values directly — a 1:1 match for the columns bound by the Java
/// PreparedStatement — and simply returns the generated id for the caller to store once
/// that wiring exists.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Value};

pub const SQL: &str = "INSERT INTO ffb_games_info \
    (scheduled, started, finished, coach_home, team_home_id, team_home_name, \
     coach_away, team_away_id, team_away_name, half, turn, home_playing, status, testing, admin_mode) \
    VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";

/// Manual epoch-millis -> MySQL DATETIME conversion (inverse of the days_from_civil
/// helper used by the read-side query files; mysql_common has no chrono/time feature
/// enabled in this workspace).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn millis_to_value(millis: i64) -> Value {
    let days = millis.div_euclid(86_400_000);
    let rem = millis.rem_euclid(86_400_000);
    let (y, mo, d) = civil_from_days(days);
    let h = (rem / 3_600_000) as u8;
    let mi = ((rem % 3_600_000) / 60_000) as u8;
    let s = ((rem % 60_000) / 1000) as u8;
    let micro = ((rem % 1000) * 1000) as u32;
    Value::Date(y as u16, mo as u8, d as u8, h, mi, s, micro)
}

fn opt_millis_to_value(millis: Option<i64>) -> Value {
    match millis {
        Some(m) => millis_to_value(m),
        None => Value::NULL,
    }
}

fn opt_str_to_value(s: Option<&str>) -> Value {
    match s {
        Some(s) => Value::Bytes(s.as_bytes().to_vec()),
        None => Value::NULL,
    }
}

pub struct DbGamesInfoInsertQuery;

impl DbGamesInfoInsertQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::GAMES_INFO_INSERT_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Inserts a new game_info row and returns the generated game id.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        &self,
        conn: &mut Conn,
        scheduled: Option<i64>,
        started: Option<i64>,
        finished: Option<i64>,
        coach_home: Option<&str>,
        team_home_id: Option<&str>,
        team_home_name: Option<&str>,
        coach_away: Option<&str>,
        team_away_id: Option<&str>,
        team_away_name: Option<&str>,
        half: i8,
        turn: i8,
        home_playing: bool,
        status: &str,
        testing: bool,
        admin_mode: bool,
    ) -> Result<i64, DbError> {
        // mysql_async's tuple `Params` impls only go up to 12 elements; this statement
        // binds 15, so the params are passed as a positional `Vec<Value>` instead.
        let params: Vec<Value> = vec![
            opt_millis_to_value(scheduled),
            opt_millis_to_value(started),
            opt_millis_to_value(finished),
            opt_str_to_value(coach_home),
            opt_str_to_value(team_home_id),
            opt_str_to_value(team_home_name),
            opt_str_to_value(coach_away),
            opt_str_to_value(team_away_id),
            opt_str_to_value(team_away_name),
            Value::Int(half as i64),
            Value::Int(turn as i64),
            Value::Int(home_playing as i64),
            Value::Bytes(status.as_bytes().to_vec()),
            Value::Int(testing as i64),
            Value::Int(admin_mode as i64),
        ];
        conn.exec_drop(SQL, params).await?;
        Ok(conn.last_insert_id().unwrap_or(0) as i64)
    }
}

impl Default for DbGamesInfoInsertQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbGamesInfoInsertQuery::new();
        assert_eq!(q.get_id(), DbStatementId::GAMES_INFO_INSERT_QUERY);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_games_info"));
    }

    #[test]
    fn sql_has_all_placeholders() {
        assert_eq!(SQL.matches('?').count(), 15);
    }

    #[test]
    fn millis_round_trips_through_civil_date() {
        let v = millis_to_value(0);
        assert_eq!(v, Value::Date(1970, 1, 1, 0, 0, 0, 0));
    }

    #[test]
    fn opt_millis_none_is_null() {
        assert_eq!(opt_millis_to_value(None), Value::NULL);
    }
}
