/// 1:1 translation of com.fumbbl.ffb.server.db.IDbStatementFactory.
use crate::db::db_statement::DbStatement;
use crate::db::db_statement_id::DbStatementId;

pub trait IDbStatementFactory {
    fn get_statement(&self, statement_id: DbStatementId) -> Option<&dyn DbStatement>;
}
