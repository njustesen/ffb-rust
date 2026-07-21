/// 1:1 translation of com.fumbbl.ffb.server.db.query.DbPasswordForCoachQuery.
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

pub const SQL: &str = "SELECT password FROM ffb_coaches WHERE name = ?";

pub struct DbPasswordForCoachQuery;

impl DbPasswordForCoachQuery {
    pub fn new() -> Self {
        Self
    }

    pub fn get_id(&self) -> DbStatementId {
        DbStatementId::PASSWORD_FOR_COACH_QUERY
    }

    /// prepare() is a JDBC artifact — mysql_async does not need it.
    // pub fn prepare(&mut self, conn: &mut Conn) { /* no-op */ }

    pub async fn execute(&self, conn: &mut Conn, coach: &str) -> Result<Option<String>, DbError> {
        conn.exec_first(SQL, (coach,)).await
    }
}

impl Default for DbPasswordForCoachQuery {
    fn default() -> Self {
        Self::new()
    }
}
