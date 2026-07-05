use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSelectBlitzTarget.java`.
#[derive(Debug, Clone)]
pub struct ReportSelectBlitzTarget {
    pub attacker: Option<String>,
    pub defender: Option<String>,
}

impl ReportSelectBlitzTarget {
    pub fn new(attacker: Option<String>, defender: Option<String>) -> Self {
        Self { attacker, defender }
    }

    pub fn get_attacker(&self) -> Option<&str> { self.attacker.as_deref() }
    pub fn get_defender(&self) -> Option<&str> { self.defender.as_deref() }
}

impl IReport for ReportSelectBlitzTarget {
    fn get_id(&self) -> ReportId { ReportId::SELECT_BLITZ_TARGET }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSelectBlitzTarget {
        ReportSelectBlitzTarget::new(Some("a1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SELECT_BLITZ_TARGET); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "selectBlitzTarget"); }

    #[test]
    fn get_attacker() { assert_eq!(make().get_attacker(), Some("a1")); }

    #[test]
    fn get_defender() { assert_eq!(make().get_defender(), Some("d1")); }
}
