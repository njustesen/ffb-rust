use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThenIStartedBlastin.java`.
#[derive(Debug, Clone)]
pub struct ReportThenIStartedBlastin {
    pub player_id: Option<String>,
    pub target_player_id: Option<String>,
    pub roll: i32,
    pub success: bool,
    pub fumble: bool,
}

impl ReportThenIStartedBlastin {
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

impl IReport for ReportThenIStartedBlastin {
    fn get_id(&self) -> ReportId { ReportId::THEN_I_STARTED_BLASTIN }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThenIStartedBlastin {
        ReportThenIStartedBlastin::new(Some("p1".into()), Some("t1".into()), 4, true, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::THEN_I_STARTED_BLASTIN); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "thenIStartedBlastin"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 4); }

    #[test]
    fn is_success() { assert!(make().is_success()); }

    #[test]
    fn is_fumble() { assert!(!make().is_fumble()); }
}
