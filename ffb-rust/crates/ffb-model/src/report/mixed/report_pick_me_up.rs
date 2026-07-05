use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPickMeUp.java`.
#[derive(Debug, Clone)]
pub struct ReportPickMeUp {
    pub player_id: Option<String>,
    pub success: bool,
    pub roll: i32,
}

impl ReportPickMeUp {
    pub fn new(player_id: Option<String>, roll: i32, success: bool) -> Self {
        Self { player_id, success, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_success(&self) -> bool { self.success }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportPickMeUp {
    fn get_id(&self) -> ReportId { ReportId::PICK_ME_UP }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPickMeUp {
        ReportPickMeUp::new(Some("p1".into()), 5, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PICK_ME_UP); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "pickMeUp"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 5); }

    #[test]
    fn is_success() { assert!(make().is_success()); }
}
