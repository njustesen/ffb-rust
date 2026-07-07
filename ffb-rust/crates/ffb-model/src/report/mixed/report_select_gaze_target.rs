use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSelectGazeTarget.java`.
#[derive(Debug, Clone)]
pub struct ReportSelectGazeTarget {
    pub attacker: Option<String>,
    pub defender: Option<String>,
}

impl ReportSelectGazeTarget {
    pub fn new(attacker: Option<String>, defender: Option<String>) -> Self {
        Self { attacker, defender }
    }

    pub fn get_attacker(&self) -> Option<&str> { self.attacker.as_deref() }
    pub fn get_defender(&self) -> Option<&str> { self.defender.as_deref() }
}

impl IReport for ReportSelectGazeTarget {
    fn get_id(&self) -> ReportId { ReportId::SELECT_GAZE_TARGET }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSelectGazeTarget {
        ReportSelectGazeTarget::new(Some("a1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SELECT_GAZE_TARGET); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "selectGazeTarget"); }

    #[test]
    fn get_attacker() { assert_eq!(make().get_attacker(), Some("a1")); }

    #[test]
    fn get_defender() { assert_eq!(make().get_defender(), Some("d1")); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }
}
