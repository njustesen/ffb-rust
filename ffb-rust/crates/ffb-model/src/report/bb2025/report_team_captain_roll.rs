use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTeamCaptainRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTeamCaptainRoll {
    pub team_id: String,
    pub minimum_roll: i32,
    pub roll: i32,
    pub successful: bool,
}

impl ReportTeamCaptainRoll {
    pub fn new(team_id: String, minimum_roll: i32, roll: i32, successful: bool) -> Self {
        Self { team_id, minimum_roll, roll, successful }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
}

impl IReport for ReportTeamCaptainRoll {
    fn get_id(&self) -> ReportId { ReportId::TEAM_CAPTAIN_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTeamCaptainRoll {
        ReportTeamCaptainRoll::new("team1".into(), 4, 5, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TEAM_CAPTAIN_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "teamCaptainRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_minimum_roll(), 4);
        assert!(r.is_successful());
    }
}
