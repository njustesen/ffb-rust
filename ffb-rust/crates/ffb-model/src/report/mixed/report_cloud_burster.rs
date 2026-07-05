use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCloudBurster.java`.
#[derive(Debug, Clone)]
pub struct ReportCloudBurster {
    pub thrower_id: Option<String>,
    pub interceptor_id: Option<String>,
    pub thrower_team_id: Option<String>,
}

impl ReportCloudBurster {
    pub fn new(
        thrower_id: Option<String>,
        interceptor_id: Option<String>,
        thrower_team_id: Option<String>,
    ) -> Self {
        Self { thrower_id, interceptor_id, thrower_team_id }
    }

    pub fn get_thrower_id(&self) -> Option<&str> { self.thrower_id.as_deref() }
    pub fn get_interceptor_id(&self) -> Option<&str> { self.interceptor_id.as_deref() }
    pub fn get_thrower_team_id(&self) -> Option<&str> { self.thrower_team_id.as_deref() }
}

impl IReport for ReportCloudBurster {
    fn get_id(&self) -> ReportId { ReportId::CLOUD_BURSTER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCloudBurster {
        ReportCloudBurster::new(Some("t1".into()), Some("i1".into()), Some("team1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::CLOUD_BURSTER); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "cloudBurster"); }

    #[test]
    fn get_thrower_id() { assert_eq!(make().get_thrower_id(), Some("t1")); }
}
