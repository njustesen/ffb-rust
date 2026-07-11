/// 1:1 translation of com.fumbbl.ffb.server.db.IDbUpdateParameterList.
pub trait IDbUpdateParameterList {
    fn get_parameters(&self) -> Vec<Box<dyn super::i_db_update_parameter::IDbUpdateParameter>>;
}
