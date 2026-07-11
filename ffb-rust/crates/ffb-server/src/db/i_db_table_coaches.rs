/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTableCoaches.
pub trait IDbTableCoaches {
    const TABLE_NAME: &'static str = "ffb_coaches";
    const COLUMN_NAME: &'static str = "name";
    const COLUMN_PASSWORD: &'static str = "password";
}
