use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportIndomitable.java`.
#[derive(Debug, Clone)]
pub struct ReportIndomitable {
    pub player_id: Option<String>,
    pub defender_id: Option<String>,
}

impl ReportIndomitable {
    pub fn new(player_id: Option<String>, defender_id: Option<String>) -> Self {
        Self { player_id, defender_id }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportIndomitable {
    fn get_id(&self) -> ReportId { ReportId::INDOMITABLE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportIndomitable {
        ReportIndomitable::new(Some("p1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::INDOMITABLE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "indomitable"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }
}
