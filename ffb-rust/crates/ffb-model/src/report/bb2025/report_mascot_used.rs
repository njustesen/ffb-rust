use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportMascotUsed.java`.
#[derive(Debug, Clone)]
pub struct ReportMascotUsed {
    pub team_id: String,
    pub minimum_roll: i32,
    pub roll: i32,
    pub successful: bool,
    pub fallback: bool,
}

impl ReportMascotUsed {
    pub fn new(team_id: String, minimum_roll: i32, roll: i32, successful: bool, fallback: bool) -> Self {
        Self { team_id, minimum_roll, roll, successful, fallback }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_fallback(&self) -> bool { self.fallback }
}

impl IReport for ReportMascotUsed {
    fn get_id(&self) -> ReportId { ReportId::MASCOT_USED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportMascotUsed {
        ReportMascotUsed::new("team1".into(), 4, 5, true, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::MASCOT_USED);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "mascotUsed");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_successful());
        assert!(!r.is_fallback());
    }

    #[test]
    fn minimum_roll_and_roll() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 4);
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn fallback_and_unsuccessful() {
        let r = ReportMascotUsed::new("team2".into(), 5, 3, false, true);
        assert!(!r.is_successful());
        assert!(r.is_fallback());
        assert_eq!(r.get_team_id(), "team2");
    }
}
