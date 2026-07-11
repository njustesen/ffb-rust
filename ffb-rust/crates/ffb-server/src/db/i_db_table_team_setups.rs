/// 1:1 translation of com.fumbbl.ffb.server.db.IDbTableTeamSetups.
pub trait IDbTableTeamSetups {
    const LENGTH_NAME: usize = 40;
    const TABLE_NAME: &'static str = "ffb_team_setups";
    const COLUMN_TEAM_ID: &'static str = "team_id";
    const COLUMN_NAME: &'static str = "name";
    /// Template: player_nr_$1 through player_nr_$11
    const COLUMN_PLAYER_NR: &'static str = "player_nr_$1";
    /// Template: coordinate_$1_x through coordinate_$11_x
    const COLUMN_COORDINATE_X: &'static str = "coordinate_$1_x";
    /// Template: coordinate_$1_y through coordinate_$11_y
    const COLUMN_COORDINATE_Y: &'static str = "coordinate_$1_y";
}
