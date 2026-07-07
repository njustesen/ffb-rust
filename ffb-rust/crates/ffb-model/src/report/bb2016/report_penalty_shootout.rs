use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPenaltyShootout.java`.
#[derive(Debug, Clone)]
pub struct ReportPenaltyShootout {
    pub roll_home: i32,
    pub re_rolls_left_home: i32,
    pub roll_away: i32,
    pub re_rolls_left_away: i32,
}

impl ReportPenaltyShootout {
    pub fn new(roll_home: i32, re_rolls_left_home: i32, roll_away: i32, re_rolls_left_away: i32) -> Self {
        Self { roll_home, re_rolls_left_home, roll_away, re_rolls_left_away }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_re_rolls_left_home(&self) -> i32 { self.re_rolls_left_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_re_rolls_left_away(&self) -> i32 { self.re_rolls_left_away }
}

impl IReport for ReportPenaltyShootout {
    fn get_id(&self) -> ReportId { ReportId::PENALTY_SHOOTOUT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPenaltyShootout {
        ReportPenaltyShootout::new(4, 2, 3, 1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PENALTY_SHOOTOUT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "penaltyShootout");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_re_rolls_left_home(), 2);
        assert_eq!(r.get_roll_away(), 3);
        assert_eq!(r.get_re_rolls_left_away(), 1);
    }

    #[test]
    fn zero_rerolls_left() {
        let r = ReportPenaltyShootout::new(5, 0, 6, 0);
        assert_eq!(r.get_re_rolls_left_home(), 0);
        assert_eq!(r.get_re_rolls_left_away(), 0);
    }

    #[test]
    fn asymmetric_values() {
        let r = ReportPenaltyShootout::new(1, 3, 6, 0);
        assert_eq!(r.get_roll_home(), 1);
        assert_eq!(r.get_roll_away(), 6);
        assert_eq!(r.get_re_rolls_left_home(), 3);
    }
}
