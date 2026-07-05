use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSwarmingRoll.java`.
/// `roll` and `limit` default to `-1` (sentinel "not set"), matching Java.
#[derive(Debug, Clone)]
pub struct ReportSwarmingRoll {
    pub team_id: Option<String>,
    pub amount: i32,
    pub roll: i32,
    pub limit: i32,
}

impl ReportSwarmingRoll {
    pub fn new(team_id: Option<String>, amount: i32, roll: i32, limit: i32) -> Self {
        Self { team_id, amount, roll, limit }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_limit(&self) -> i32 { self.limit }
}

impl IReport for ReportSwarmingRoll {
    fn get_id(&self) -> ReportId { ReportId::SWARMING_PLAYERS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSwarmingRoll {
        ReportSwarmingRoll::new(Some("team1".into()), 3, 2, 4)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SWARMING_PLAYERS_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "swarmingPlayersRoll"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 2); }

    #[test]
    fn get_limit() { assert_eq!(make().get_limit(), 4); }

    #[test]
    fn default_sentinels() {
        let r = ReportSwarmingRoll::new(None, 0, -1, -1);
        assert_eq!(r.get_roll(), -1);
        assert_eq!(r.get_limit(), -1);
    }
}
