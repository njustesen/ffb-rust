/// 1:1 translation of com.fumbbl.ffb.server.db.IDbUpdateParameter.
pub trait IDbUpdateParameter {
    fn get_updated_rows(&self) -> i32;
    fn execute_update(&mut self) -> Result<(), String>;
}
