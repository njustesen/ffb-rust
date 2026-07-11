/// 1:1 translation of com.fumbbl.ffb.server.db.DbStatement.
use crate::db::db_statement_id::DbStatementId;

pub trait DbStatement {
    fn get_id(&self) -> DbStatementId;
    fn prepare(&mut self) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct _TestStmt;
    impl DbStatement for _TestStmt {
        fn get_id(&self) -> DbStatementId { DbStatementId::GAMES_INFO_UPDATE }
        fn prepare(&mut self) -> Result<(), String> { Ok(()) }
    }

    #[test]
    fn trait_object() {
        let s: &dyn DbStatement = &_TestStmt;
        assert_eq!(s.get_id(), DbStatementId::GAMES_INFO_UPDATE);
    }
}
