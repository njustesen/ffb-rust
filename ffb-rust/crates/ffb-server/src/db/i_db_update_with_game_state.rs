/// 1:1 translation of com.fumbbl.ffb.server.db.IDbUpdateWithGameState.
pub trait IDbUpdateWithGameState {
    fn get_id(&self) -> i64;
}
