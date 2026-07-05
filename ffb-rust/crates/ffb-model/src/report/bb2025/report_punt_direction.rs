use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::Direction;

/// 1:1 translation of `ReportPuntDirection.java`.
#[derive(Debug, Clone)]
pub struct ReportPuntDirection {
    pub direction: Option<Direction>,
    pub direction_roll: i32,
    pub player_id: String,
    pub out_of_bounds: bool,
}

impl ReportPuntDirection {
    pub fn new(direction: Option<Direction>, direction_roll: i32, player_id: String, out_of_bounds: bool) -> Self {
        Self { direction, direction_roll, player_id, out_of_bounds }
    }

    pub fn get_direction(&self) -> Option<Direction> { self.direction }
    pub fn get_direction_roll(&self) -> i32 { self.direction_roll }
    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }
}

impl IReport for ReportPuntDirection {
    fn get_id(&self) -> ReportId { ReportId::PUNT_DIRECTION_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPuntDirection {
        ReportPuntDirection::new(Some(Direction::North), 3, "p1".into(), false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PUNT_DIRECTION_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "puntDirectionRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Some(Direction::North));
        assert_eq!(r.get_direction_roll(), 3);
        assert!(!r.is_out_of_bounds());
    }
}
