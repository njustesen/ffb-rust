use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTimeoutEnforced.java`.
#[derive(Debug, Clone)]
pub struct ReportTimeoutEnforced {
    pub coach: String,
}

impl ReportTimeoutEnforced {
    pub fn new(coach: String) -> Self {
        Self { coach }
    }

    pub fn get_coach(&self) -> &str { &self.coach }
}

impl IReport for ReportTimeoutEnforced {
    fn get_id(&self) -> ReportId { ReportId::TIMEOUT_ENFORCED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTimeoutEnforced {
        ReportTimeoutEnforced::new("Coach McCoach".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TIMEOUT_ENFORCED);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "timeoutEnforced");
    }

    #[test]
    fn get_coach() {
        assert_eq!(make().get_coach(), "Coach McCoach");
    }
}
