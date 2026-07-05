use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStallerDetected.java`.
#[derive(Debug, Clone)]
pub struct ReportStallerDetected {
    /// `fPlayerId`
    pub player_id: Option<String>,
}

impl ReportStallerDetected {
    pub fn new(player_id: Option<String>) -> Self {
        Self { player_id }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IReport for ReportStallerDetected {
    fn get_id(&self) -> ReportId { ReportId::STALLER_DETECTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStallerDetected {
        ReportStallerDetected::new(Some("p1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::STALLER_DETECTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "stallerDetected"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }
}
