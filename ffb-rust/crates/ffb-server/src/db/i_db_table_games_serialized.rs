/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTableGamesSerialized.
pub trait IDbTableGamesSerialized {
    const TABLE_NAME: &'static str = "ffb_games_serialized";
    const COLUMN_ID: &'static str = "id";
    const COLUMN_SERIALIZED: &'static str = "serialized";
}
