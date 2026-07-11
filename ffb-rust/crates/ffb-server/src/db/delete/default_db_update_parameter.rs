/// 1:1 translation of com.fumbbl.ffb.server.db.delete.DefaultDbUpdateParameter.
/// Abstract base for all DB delete/update parameter objects.
/// Subclasses store their own SQL and typed fields; this base tracks updated_rows
/// and provides empty after-commit/rollback hooks.
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

use crate::db::i_db_update_parameter::IDbUpdateParameter;

pub struct DefaultDbUpdateParameter {
    updated_rows: i32,
}

impl DefaultDbUpdateParameter {
    pub fn new() -> Self {
        Self { updated_rows: 0 }
    }

    /// Called by the owning DbTransaction after a successful commit.
    /// Override in subclasses if needed.
    pub fn do_after_commit(&self) {
        // empty hook — override in subclasses
    }

    /// Called by the owning DbTransaction after a rollback.
    /// Override in subclasses if needed.
    pub fn do_after_rollback(&self) {
        // empty hook — override in subclasses
    }

    /// Async entry-point used by concrete parameter subclasses.
    /// Java: fUpdatedRows = getDbUpdateStatement(pServer).execute(this);
    /// Concrete subclasses call this after running conn.exec_drop() to store
    /// the affected-row count returned by conn.affected_rows().
    pub fn set_updated_rows(&mut self, rows: i32) {
        self.updated_rows = rows;
    }
}

impl Default for DefaultDbUpdateParameter {
    fn default() -> Self {
        Self::new()
    }
}

impl IDbUpdateParameter for DefaultDbUpdateParameter {
    fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// The synchronous execute_update() path is a JDBC artifact and is not used
    /// in the async mysql_async port. Concrete parameter structs expose their own
    /// `pub async fn execute(&self, conn: &mut Conn, ...) -> Result<u64, DbError>`
    /// method instead. This implementation is left as unreachable so that the
    /// trait bound is satisfied without panicking on accidental calls.
    fn execute_update(&mut self) -> Result<(), String> {
        // Phase ZV: replaced by async execute() on concrete parameter structs.
        // This synchronous path is never reached in the mysql_async port.
        Err("execute_update() is a JDBC artifact; use async execute() instead".to_string())
    }
}

/// Integration-test skeleton — does not run without a live DB but must compile.
/// Run with: cargo test -p ffb-server -- db::delete::default_db_update_parameter
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_default_rows_zero() {
        let p = DefaultDbUpdateParameter::new();
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn set_updated_rows_roundtrip() {
        let mut p = DefaultDbUpdateParameter::new();
        p.set_updated_rows(42);
        assert_eq!(p.get_updated_rows(), 42);
    }

    #[test]
    fn do_after_commit_is_noop() {
        let p = DefaultDbUpdateParameter::new();
        p.do_after_commit(); // must not panic
    }

    #[test]
    fn do_after_rollback_is_noop() {
        let p = DefaultDbUpdateParameter::new();
        p.do_after_rollback(); // must not panic
    }

    #[test]
    fn execute_update_returns_err() {
        let mut p = DefaultDbUpdateParameter::new();
        assert!(p.execute_update().is_err());
    }

    /// Integration-test skeleton: shows how a concrete subtype would call execute().
    /// Not run automatically (no live DB). Kept here so the async API compiles.
    #[allow(dead_code)]
    async fn _integration_skeleton(conn: &mut Conn) -> Result<u64, DbError> {
        // Example of what a concrete subtype does:
        //   conn.exec_drop("DELETE FROM ...", (param1,)).await?;
        //   Ok(conn.affected_rows())
        let _ = conn;
        Ok(0)
    }
}
