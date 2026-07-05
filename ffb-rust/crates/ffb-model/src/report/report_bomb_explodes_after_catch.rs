use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBombExplodesAfterCatch.java`.
#[derive(Debug, Clone)]
pub struct ReportBombExplodesAfterCatch {
    pub catcher_id: String,
    pub explodes: bool,
    pub roll: i32,
}

impl ReportBombExplodesAfterCatch {
    pub fn new(catcher_id: String, explodes: bool, roll: i32) -> Self {
        Self { catcher_id, explodes, roll }
    }

    pub fn get_catcher_id(&self) -> &str { &self.catcher_id }
    pub fn explodes(&self) -> bool { self.explodes }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportBombExplodesAfterCatch {
    fn get_id(&self) -> ReportId { ReportId::BOMB_EXPLODES_AFTER_CATCH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBombExplodesAfterCatch {
        ReportBombExplodesAfterCatch::new("p1".into(), true, 5)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BOMB_EXPLODES_AFTER_CATCH);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "bombExplodesAfterCatch");
    }

    #[test]
    fn get_catcher_id() {
        assert_eq!(make().get_catcher_id(), "p1");
    }
}
