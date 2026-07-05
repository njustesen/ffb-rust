use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopDirection.java`.
#[derive(Debug, Clone)]
pub struct ReportSwoopDirection {
    pub direction: Option<Direction>,
    pub direction_roll: i32,
    pub player_id: String,
    pub out_of_bounds: bool,
}

impl ReportSwoopDirection {
    pub fn new(direction: Option<Direction>, direction_roll: i32, player_id: String, out_of_bounds: bool) -> Self {
        Self { direction, direction_roll, player_id, out_of_bounds }
    }

    pub fn get_direction(&self) -> Option<Direction> { self.direction }
    pub fn get_direction_roll(&self) -> i32 { self.direction_roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }
}

impl IReport for ReportSwoopDirection {
    fn get_id(&self) -> ReportId { ReportId::SWOOP_DIRECTION_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSwoopDirection {
        ReportSwoopDirection::new(Some(Direction::East), 5, "p1".into(), false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SWOOP_DIRECTION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "swoopDirectionRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Some(Direction::East));
        assert_eq!(r.get_player_id(), "p1");
    }
}
