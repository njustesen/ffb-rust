use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDodgySnackRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportDodgySnackRoll {
    pub roll: i32,
    pub player_id: String,
}

impl ReportDodgySnackRoll {
    pub fn new(roll: i32, player_id: String) -> Self {
        Self { roll, player_id }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }
}

impl IReport for ReportDodgySnackRoll {
    fn get_id(&self) -> ReportId { ReportId::DODGY_SNACK_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDodgySnackRoll {
        ReportDodgySnackRoll::new(4, "p1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DODGY_SNACK_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "dodgySnackRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll(), 4);
        assert_eq!(r.get_player_id(), "p1");
    }
}
