/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTableUserSettings.
pub trait IDbTableUserSettings {
    const TABLE_NAME: &'static str = "ffb_user_settings";
    const COLUMN_COACH: &'static str = "coach";
    const COLUMN_NAME: &'static str = "name";
    const COLUMN_VALUE: &'static str = "value";
}
