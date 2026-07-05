use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrownKeg.java`.
#[derive(Debug, Clone)]
pub struct ReportThrownKeg {
    pub player_id: Option<String>,
    pub target_player_id: Option<String>,
    pub roll: i32,
    pub success: bool,
    pub fumble: bool,
}

impl ReportThrownKeg {
    pub fn new(
        player_id: Option<String>,
        target_player_id: Option<String>,
        roll: i32,
        success: bool,
        fumble: bool,
    ) -> Self {
        Self { player_id, target_player_id, roll, success, fumble }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_target_player_id(&self) -> Option<&str> { self.target_player_id.as_deref() }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_success(&self) -> bool { self.success }
    pub fn is_fumble(&self) -> bool { self.fumble }
}

impl IReport for ReportThrownKeg {
    fn get_id(&self) -> ReportId { ReportId::THROWN_KEG }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrownKeg {
        ReportThrownKeg::new(Some("p1".into()), Some("t1".into()), 3, true, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THROWN_KEG); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "thrownKeg"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 3); }

    #[test]
    fn is_success() { assert!(make().is_success()); }

    #[test]
    fn is_fumble() { assert!(!make().is_fumble()); }
}
