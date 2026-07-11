/// 1:1 translation of com.fumbbl.ffb.server.db.DbUpdateStatement.
use crate::db::db_statement::DbStatement;
use crate::db::i_db_update_parameter::IDbUpdateParameter;

pub trait DbUpdateStatement: DbStatement {
    fn execute(&self, update_parameter: &dyn IDbUpdateParameter) -> Result<i32, String>;
    fn to_string_param(&self, update_parameter: &dyn IDbUpdateParameter) -> Result<String, String>;
}
