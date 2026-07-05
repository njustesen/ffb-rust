use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSwarmingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportSwarmingRoll {
    pub team_id: String,
    pub roll: i32,
}

impl ReportSwarmingRoll {
    pub fn new(team_id: String, roll: i32) -> Self {
        Self { team_id, roll }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportSwarmingRoll {
    fn get_id(&self) -> ReportId { ReportId::SWARMING_PLAYERS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportSwarmingRoll::new("t1".into(), 2).get_id(), ReportId::SWARMING_PLAYERS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportSwarmingRoll::new("t1".into(), 2).get_name(), "swarmingPlayersRoll");
    }

    #[test]
    fn fields() {
        let r = ReportSwarmingRoll::new("t1".into(), 2);
        assert_eq!(r.get_team_id(), "t1");
        assert_eq!(r.get_roll(), 2);
    }
}
