use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportInducement.java`.
/// `InducementType` is serialised as a `String` (its name).
#[derive(Debug, Clone)]
pub struct ReportInducement {
    /// Translated from `fTeamId`.
    pub team_id: String,
    /// Translated from `fInducementType` (InducementType → String).
    pub inducement_type: String,
    /// Translated from `fValue`.
    pub value: i32,
}

impl ReportInducement {
    pub fn new(team_id: String, inducement_type: String, value: i32) -> Self {
        Self { team_id, inducement_type, value }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_inducement_type(&self) -> &str {
        &self.inducement_type
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}

impl IReport for ReportInducement {
    fn get_id(&self) -> ReportId {
        ReportId::INDUCEMENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInducement {
        ReportInducement::new("team1".into(), "WIZARD".into(), 150000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INDUCEMENT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "inducement");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_inducement_type(), "WIZARD");
        assert_eq!(r.get_value(), 150000);
    }

    #[test]
    fn different_team() {
        let r = ReportInducement::new("team2".into(), "BRIBE".into(), 100000);
        assert_eq!(r.get_team_id(), "team2");
        assert_eq!(r.get_inducement_type(), "BRIBE");
    }

    #[test]
    fn zero_value() {
        let r = ReportInducement::new("team1".into(), "FREE".into(), 0);
        assert_eq!(r.get_value(), 0);
    }
}
