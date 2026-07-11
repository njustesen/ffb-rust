/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbTeamSetupsQuery.
///
/// NOTE: ffb-model's TeamSetup (crate::model::team_setup) does not mirror the Java
/// TeamSetup's per-position FieldCoordinate map, so a dedicated row struct is returned
/// here instead, matching the Java ResultSet column reads (player_nr_$i,
/// coordinate_$i_x, coordinate_$i_y for i in 1..=11) 1:1. Wiring this into the richer
/// ffb-model TeamSetup is deferred to a later phase.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Row};

pub const SQL: &str = "SELECT * FROM ffb_team_setups WHERE (team_id = ? AND name = ?)";

/// One occupied square from a stored team setup (player_nr > 0 only, matching Java's
/// `if (playerNr > 0)` guard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamSetupPosition {
    pub player_nr: i32,
    pub coordinate_x: i32,
    pub coordinate_y: i32,
}

pub struct DbTeamSetupsQuery;

impl DbTeamSetupsQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::TEAM_SETUPS_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    /// Returns the occupied positions for the given team + setup name.
    pub async fn execute(
        &self,
        conn: &mut Conn,
        team_id: &str,
        setup_name: &str,
    ) -> Result<Vec<TeamSetupPosition>, DbError> {
        let rows: Vec<Row> = conn.exec(SQL, (team_id, setup_name)).await?;
        let mut positions = Vec::new();
        for mut row in rows {
            for i in 0..11usize {
                let player_nr: i8 = row.take(2 + 3 * i).unwrap_or(0);
                let coordinate_x: i8 = row.take(3 + 3 * i).unwrap_or(0);
                let coordinate_y: i8 = row.take(4 + 3 * i).unwrap_or(0);
                if player_nr > 0 {
                    positions.push(TeamSetupPosition {
                        player_nr: player_nr as i32,
                        coordinate_x: coordinate_x as i32,
                        coordinate_y: coordinate_y as i32,
                    });
                }
            }
        }
        Ok(positions)
    }
}

impl Default for DbTeamSetupsQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let q = DbTeamSetupsQuery::new();
        assert_eq!(q.get_id(), DbStatementId::TEAM_SETUPS_QUERY);
    }

    #[test]
    fn sql_constant() {
        assert!(SQL.contains("ffb_team_setups"));
    }

    #[test]
    fn sql_has_where_clause() {
        assert!(SQL.contains("team_id = ?"));
        assert!(SQL.contains("name = ?"));
    }
}
