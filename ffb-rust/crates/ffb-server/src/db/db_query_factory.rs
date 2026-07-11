/// 1:1 translation of com.fumbbl.ffb.server.db.DbQueryFactory.
///
/// Java's constructor takes a FantasyFootballServer and registers ~10 concrete
/// DbStatement query objects (DbAdminListByIdQuery, DbGamesSerializedQuery, ...) into a
/// `Map<DbStatementId, DbStatement>`. Rust has no `FantasyFootballServer` struct yet in
/// this phase, and the concrete query types each have a distinct `execute()` signature
/// (no shared dyn-compatible trait exists for them), so a real heterogeneous statement
/// registry isn't possible without a larger refactor. Instead this keeps a
/// `HashMap<DbStatementId, DbStatementId>` as an honest, testable analog of "is this
/// statement id registered" — `get_statement()` returns `Option<DbStatementId>` confirming
/// registration/lookup rather than a boxed statement object.
use crate::db::db_connection_manager::DbConnectionManager;
use crate::db::db_statement_id::DbStatementId;
use mysql_async::Conn;
use std::collections::HashMap;

pub struct DbQueryFactory {
    db_connection_manager: DbConnectionManager,
    db_connection: Option<Conn>,
    statement_by_id: HashMap<DbStatementId, DbStatementId>,
}

impl DbQueryFactory {
    pub fn new(db_connection_manager: DbConnectionManager) -> Self {
        let mut statement_by_id = HashMap::new();

        Self::register(&mut statement_by_id, DbStatementId::ADMIN_LIST_BY_ID_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::ADMIN_LIST_BY_STATUS_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::GAME_LIST_QUERY_OPEN_GAMES_BY_COACH);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_SERIALIZED_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::GAMES_INFO_INSERT_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::TEAM_SETUPS_QUERY_ALL_FOR_A_TEAM);
        Self::register(&mut statement_by_id, DbStatementId::TEAM_SETUPS_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::USER_SETTINGS_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::PLAYER_MARKERS_QUERY);
        Self::register(&mut statement_by_id, DbStatementId::TEST_GAME_LIST_QUERY);

        // Java: if (ServerMode.STANDALONE == getServer().getMode()) register password query.
        // is_standalone() on DbConnectionManager defaults to false pending full server-mode
        // wiring (see db_connection_manager.rs), matching Java's fumbbl-mode default.
        if db_connection_manager.is_standalone() {
            Self::register(&mut statement_by_id, DbStatementId::PASSWORD_FOR_COACH_QUERY);
        }

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

    pub async fn prepare_statements(&mut self) -> Result<(), mysql_async::Error> {
        let conn = self.db_connection_manager.open_db_connection().await?;
        self.db_connection = Some(conn);
        // Java: fDbConnection.setAutoCommit(true), then statement.prepare(fDbConnection) for
        // each registered statement. prepare() is a JDBC artifact — mysql_async statements
        // don't need it (see delete/db_games_serialized_delete.rs doc comment) — so there is
        // nothing further to do here per statement.
        Ok(())
    }

    pub async fn close_db_connection(&mut self) -> Result<(), mysql_async::Error> {
        if let Some(conn) = self.db_connection.take() {
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
        let f = DbQueryFactory::new(DbConnectionManager::new());
        assert!(f.get_statement(DbStatementId::GAMES_SERIALIZED_QUERY).is_some());
    }

    #[test]
    fn password_for_coach_query_not_registered_when_not_standalone() {
        let f = DbQueryFactory::new(DbConnectionManager::new());
        // is_standalone() defaults to false, so this statement should not be registered.
        assert!(f.get_statement(DbStatementId::PASSWORD_FOR_COACH_QUERY).is_none());
    }

    #[test]
    fn unregistered_statement_returns_none() {
        let f = DbQueryFactory::new(DbConnectionManager::new());
        assert!(f.get_statement(DbStatementId::GAMES_INFO_UPDATE).is_none());
    }
}
