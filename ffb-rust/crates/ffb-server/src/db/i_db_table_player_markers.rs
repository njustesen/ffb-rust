/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTablePlayerMarkers.
pub trait IDbTablePlayerMarkers {
    const TABLE_NAME: &'static str = "ffb_player_markers";
    const COLUMN_TEAM_ID: &'static str = "team_id";
    const COLUMN_PLAYER_ID: &'static str = "player_id";
    const COLUMN_TEXT: &'static str = "text";
    const MAX_TEXT_LENGTH: usize = 40;
}
