/// 1:1 translation of com.fumbbl.ffb.server.db.DbUpdateFactory.
///
/// Java's constructor takes a FantasyFootballServer and registers ~11 concrete
/// DbUpdateStatement objects (DbGamesSerializedInsert, DbTeamSetupsInsert, ...) into a
/// `Map<DbStatementId, DbUpdateStatement>`. As with DbQueryFactory, there is no
/// `FantasyFootballServer` in this phase and each concrete update/insert/delete statement
/// has a distinct `execute()` signature — no shared dyn-compatible trait exists for them.
/// This keeps a `HashMap<DbStatementId, DbStatementId>` as an honest, testable registry
/// analog; `get_statement()` returns `Option<DbStatementId>` confirming registration.
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::db_statement_id::DbStatementId;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};
use std::collections::HashMap;

pub struct DbUpdateFactory {
    db_connection_manager: DbConnectionManager,
    db_connection: Option<Conn>,
    statement_by_id: HashMap<DbStatementId, DbStatementId>,
}

impl DbUpdateFactory {
    pub fn new(db_connection_manager: DbConnectionManager) -> Self {
        let mut statement_by_id = HashMap::new();

        Self::register(&mut statement_by_id, DbStatementId::GAMES_SERIALIZED_INSERT);
        Self::register(&mut statement_by_id, DbStatementId::TEAM_SETUPS_INSERT);
        Self::register(&mut statement_by_id, DbStatementId::USER_SETTINGS_INSERT);
        Self::register(&mut statement_by_id, DbStatementId::USER_SETTINGS_DELETE);
        Self::register(&mut statement_by_id, DbStatementId::PLAYER_MARKERS_INSERT);
        Self::register(&mut statement_by_id, DbStatementId::PLAYER_MARKERS_DELETE);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_INFO_UPDATE);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_SERIALIZED_UPDATE);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_INFO_DELETE);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_SERIALIZED_DELETE);
        Self::register(&mut statement_by_id, DbStatementId::TEAM_SETUPS_DELETE);

        Self {
            db_connection_manager,
            db_connection: None,
            statement_by_id,
        }
    }

    fn register(statement_by_id: &mut HashMap<DbStatementId, DbStatementId>, id: DbStatementId) {
        statement_by_id.insert(id, id);
    }

    pub fn get_statement(&self, id: DbStatementId) -> Option<DbStatementId> {
        self.statement_by_id.get(&id).copied()
    }

    pub async fn prepare_statements(&mut self) -> Result<(), DbError> {
        let conn = self.db_connection_manager.open_db_connection().await?;
        self.db_connection = Some(conn);
        // Java: fDbConnection.setAutoCommit(false), then statement.prepare(fDbConnection) for
        // each registered statement. DbConnectionManager::open_db_connection() already issues
        // "SET autocommit=0" (see db_connection_manager.rs), matching Java's setAutoCommit(false).
        // prepare() itself is a JDBC artifact with no mysql_async equivalent needed.
        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), DbError> {
        if let Some(conn) = self.db_connection.as_mut() {
            conn.query_drop("COMMIT").await?;
        }
        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<(), DbError> {
        if let Some(conn) = self.db_connection.as_mut() {
            conn.query_drop("ROLLBACK").await?;
        }
        Ok(())
    }

    pub async fn close_db_connection(&mut self) -> Result<(), DbError> {
        if let Some(mut conn) = self.db_connection.take() {
            if self.db_connection_manager.is_standalone() {
                conn.query_drop("SHUTDOWN").await?;
            }
            self.db_connection_manager.close_db_connection(conn).await?;
        }
        Ok(())
    }

    pub fn get_db_connection_manager(&self) -> &DbConnectionManager {
        &self.db_connection_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let f = DbUpdateFactory::new(DbConnectionManager::new());
        assert!(f.get_statement(DbStatementId::GAMES_INFO_UPDATE).is_some());
        assert!(f.get_statement(DbStatementId::TEAM_SETUPS_DELETE).is_some());
    }

    #[test]
    fn unregistered_statement_returns_none() {
        let f = DbUpdateFactory::new(DbConnectionManager::new());
        assert!(f.get_statement(DbStatementId::PASSWORD_FOR_COACH_QUERY).is_none());
    }

    #[test]
    fn commit_and_rollback_are_no_ops_without_open_connection() {
        // No live connection has been prepared (no DB running in tests), so commit/rollback
        // must be safe no-ops rather than panicking.
        let mut f = DbUpdateFactory::new(DbConnectionManager::new());
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert!(rt.block_on(f.commit()).is_ok());
        assert!(rt.block_on(f.rollback()).is_ok());
        assert!(rt.block_on(f.close_db_connection()).is_ok());
    }
}
