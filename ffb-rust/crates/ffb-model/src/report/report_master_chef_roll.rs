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
}
