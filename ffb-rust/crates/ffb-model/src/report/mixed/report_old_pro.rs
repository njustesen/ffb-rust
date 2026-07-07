use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportOldPro.java`.
#[derive(Debug, Clone)]
pub struct ReportOldPro {
    pub player_id: Option<String>,
    pub old_value: i32,
    pub new_value: i32,
    pub self_inflicted: bool,
}

impl ReportOldPro {
    pub fn new(player_id: Option<String>, old_value: i32, new_value: i32, self_inflicted: bool) -> Self {
        Self { player_id, old_value, new_value, self_inflicted }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_old_value(&self) -> i32 { self.old_value }
    pub fn get_new_value(&self) -> i32 { self.new_value }
    pub fn is_self_inflicted(&self) -> bool { self.self_inflicted }
}

impl IReport for ReportOldPro {
    fn get_id(&self) -> ReportId { ReportId::OLD_PRO }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportOldPro {
        ReportOldPro::new(Some("p1".into()), 3, 2, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::OLD_PRO); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "oldPro"); }

    #[test]
    fn get_new_value() { assert_eq!(make().get_new_value(), 2); }

    #[test]
    fn get_old_value_and_player_id() {
        let r = make();
        assert_eq!(r.get_old_value(), 3);
        assert_eq!(r.get_player_id(), Some("p1"));
    }

    #[test]
    fn is_self_inflicted() {
        let r = ReportOldPro::new(None, 1, 0, true);
        assert!(r.is_self_inflicted());
    }
}
