use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffOfficiousRef.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffOfficiousRef {
    pub roll_home: i32,
    pub roll_away: i32,
    pub player_ids: Vec<String>,
}

impl ReportKickoffOfficiousRef {
    pub fn new(roll_home: i32, roll_away: i32, player_ids: Vec<String>) -> Self {
        Self { roll_home, roll_away, player_ids }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
}

impl IReport for ReportKickoffOfficiousRef {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_OFFICIOUS_REF }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffOfficiousRef {
        ReportKickoffOfficiousRef::new(3, 2, vec!["p1".into()])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_OFFICIOUS_REF);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffOfficiousRef");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert_eq!(r.get_player_ids().len(), 1);
    }

    #[test]
    fn roll_away_and_player_id_content() {
        let r = make();
        assert_eq!(r.get_roll_away(), 2);
        assert_eq!(r.get_player_ids()[0], "p1");
    }

    #[test]
    fn empty_player_ids() {
        let r = ReportKickoffOfficiousRef::new(5, 6, vec![]);
        assert_eq!(r.get_player_ids().len(), 0);
        assert_eq!(r.get_roll_home(), 5);
    }
}
