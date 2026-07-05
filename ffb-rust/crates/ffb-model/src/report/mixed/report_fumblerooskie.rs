use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFumblerooskie.java`.
#[derive(Debug, Clone)]
pub struct ReportFumblerooskie {
    pub player_id: Option<String>,
    pub used: bool,
}

impl ReportFumblerooskie {
    pub fn new(player_id: Option<String>, used: bool) -> Self {
        Self { player_id, used }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_used(&self) -> bool { self.used }
}

impl IReport for ReportFumblerooskie {
    fn get_id(&self) -> ReportId { ReportId::FUMBLEROOSKIE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFumblerooskie {
        ReportFumblerooskie::new(Some("p1".into()), true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::FUMBLEROOSKIE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "fumblerooskie"); }

    #[test]
    fn is_used() { assert!(make().is_used()); }
}
