use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_pickup_roll::ReportPickupRoll as BaseReportPickupRoll;

/// 1:1 translation of `ReportPickupRoll.java` (bb2025) — extends base ReportPickupRoll.
#[derive(Debug, Clone)]
pub struct ReportPickupRoll {
    pub base: BaseReportPickupRoll,
    pub secure_the_ball: bool,
}

impl ReportPickupRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        roll_modifiers: Vec<String>,
        secure_the_ball: bool,
    ) -> Self {
        Self {
            base: BaseReportPickupRoll::new(player_id, successful, roll, minimum_roll, re_rolled, roll_modifiers),
            secure_the_ball,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn is_secure_the_ball(&self) -> bool { self.secure_the_ball }
}

impl IReport for ReportPickupRoll {
    fn get_id(&self) -> ReportId { ReportId::PICK_UP_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPickupRoll {
        ReportPickupRoll::new(Some("p1".into()), true, 4, 3, false, vec![], true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PICK_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pickUpRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert!(r.is_secure_the_ball());
    }
}
