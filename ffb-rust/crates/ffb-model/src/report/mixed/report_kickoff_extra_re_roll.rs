use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffExtraReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffExtraReRoll {
    pub roll_home: i32,
    pub roll_away: i32,
    pub team_id: Option<String>,
}

impl ReportKickoffExtraReRoll {
    pub fn new(roll_home: i32, roll_away: i32, team_id: Option<String>) -> Self {
        Self { roll_home, roll_away, team_id }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IReport for ReportKickoffExtraReRoll {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_EXTRA_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffExtraReRoll {
        ReportKickoffExtraReRoll::new(4, 2, Some("team1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_EXTRA_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "extraReRoll"); }

    #[test]
    fn get_roll_home() { assert_eq!(make().get_roll_home(), 4); }
}
