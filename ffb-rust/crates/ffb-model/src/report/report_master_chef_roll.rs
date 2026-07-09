use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportMasterChefRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportMasterChefRoll {
    /// Translated from `fTeamId`.
    pub team_id: String,
    /// Translated from `fMasterChefRoll` (`int[]`).
    pub master_chef_roll: Vec<i32>,
    /// Translated from `fReRollsStolen`.
    pub re_rolls_stolen: i32,
}

impl ReportMasterChefRoll {
    pub fn new(team_id: String, master_chef_roll: Vec<i32>, re_rolls_stolen: i32) -> Self {
        Self { team_id, master_chef_roll, re_rolls_stolen }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_master_chef_roll(&self) -> &[i32] {
        &self.master_chef_roll
    }

    pub fn get_re_rolls_stolen(&self) -> i32 {
        self.re_rolls_stolen
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "masterChefRoll": self.master_chef_roll,
            "reRollsStolen": self.re_rolls_stolen,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            master_chef_roll: json["masterChefRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            re_rolls_stolen: json["reRollsStolen"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportMasterChefRoll {
    fn get_id(&self) -> ReportId {
        ReportId::MASTER_CHEF_ROLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportMasterChefRoll {
        ReportMasterChefRoll::new("team1".into(), vec![4, 5, 3], 2)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::MASTER_CHEF_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "masterChefRoll");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_master_chef_roll(), &[4, 5, 3]);
        assert_eq!(r.get_re_rolls_stolen(), 2);
    }

    #[test]
    fn different_team() {
        let r = ReportMasterChefRoll::new("team2".into(), vec![1, 2, 3], 0);
        assert_eq!(r.get_team_id(), "team2");
        assert_eq!(r.get_re_rolls_stolen(), 0);
    }

    #[test]
    fn roll_contents() {
        let r = ReportMasterChefRoll::new("team1".into(), vec![6, 6, 6], 3);
        assert_eq!(r.get_master_chef_roll(), &[6, 6, 6]);
        assert_eq!(r.get_re_rolls_stolen(), 3);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportMasterChefRoll::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.master_chef_roll, original.master_chef_roll);
        assert_eq!(restored.re_rolls_stolen, original.re_rolls_stolen);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("masterChefRoll"));
    }
}
