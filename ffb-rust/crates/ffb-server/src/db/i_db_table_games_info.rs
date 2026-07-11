/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTableGamesInfo.
pub trait IDbTableGamesInfo {
    const TABLE_NAME: &'static str = "ffb_games_info";
    const COLUMN_ID: &'static str = "id";
    const COLUMN_SCHEDULED: &'static str = "scheduled";
    const COLUMN_STARTED: &'static str = "started";
    const COLUMN_FINISHED: &'static str = "finished";
    const COLUMN_COACH_HOME: &'static str = "coach_home";
    const COLUMN_TEAM_HOME_ID: &'static str = "team_home_id";
    const COLUMN_TEAM_HOME_NAME: &'static str = "team_home_name";
    const COLUMN_COACH_AWAY: &'static str = "coach_away";
    const COLUMN_TEAM_AWAY_ID: &'static str = "team_away_id";
    const COLUMN_TEAM_AWAY_NAME: &'static str = "team_away_name";
    const COLUMN_HALF: &'static str = "half";
    const COLUMN_TURN: &'static str = "turn";
    const COLUMN_HOME_PLAYING: &'static str = "home_playing";
    const COLUMN_STATUS: &'static str = "status";
    const COLUMN_TESTING: &'static str = "testing";
    const COLUMN_ADMIN_MODE: &'static str = "admin_mode";
    const COLUMN_LAST_UPDATED: &'static str = "last_updated";
}
