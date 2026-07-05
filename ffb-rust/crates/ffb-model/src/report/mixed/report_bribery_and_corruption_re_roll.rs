use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBriberyAndCorruptionReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBriberyAndCorruptionReRoll {
    pub team_id: Option<String>,
    pub action: String,
}

impl ReportBriberyAndCorruptionReRoll {
    pub fn new(team_id: Option<String>, action: String) -> Self {
        Self { team_id, action }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_action(&self) -> &str { &self.action }
}

impl IReport for ReportBriberyAndCorruptionReRoll {
    fn get_id(&self) -> ReportId { ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBriberyAndCorruptionReRoll {
        ReportBriberyAndCorruptionReRoll::new(Some("team1".into()), "USE".into())
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "briberyAndCorruptionReRoll"); }

    #[test]
    fn get_action() { assert_eq!(make().get_action(), "USE"); }
}
