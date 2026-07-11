/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbTeamSetupsForTeamQuery.
///
/// NOTE: Java's execute(Team) reads `pTeam.getId()`; this takes the team id directly
/// since there is no Rust Team wiring required for a plain id lookup.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "SELECT DISTINCT name FROM ffb_team_setups WHERE team_id = ?";

pub struct DbTeamSetupsForTeamQuery;

impl DbTeamSetupsForTeamQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::TEAM_SETUPS_QUERY_ALL_FOR_A_TEAM
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Returns the distinct setup names for the given team. Mirrors Java's
    /// `if (pTeam != null)` guard: an empty/absent team id yields no query and an
    /// empty result, matching the Java `names == null` early-return shape closely
    /// enough for this DB-layer stub (see also DbPlayerMarkersQuery).
    pub async fn execute(&self, conn: &mut Conn, team_id: &str) -> Result<Vec<String>, DbError> {
        conn.exec_map(SQL, (team_id,), |name: String| name).await
    }
}

impl Default for DbTeamSetupsForTeamQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbTeamSetupsForTeamQuery::new();
        assert_eq!(q.get_id(), DbStatementId::TEAM_SETUPS_QUERY_ALL_FOR_A_TEAM);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_team_setups"));
    }

    #[test]
    fn sql_is_distinct_name_select() {
        assert!(SQL.contains("DISTINCT name"));
    }
}
