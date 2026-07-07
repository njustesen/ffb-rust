use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;

/// 1:1 translation of `ReportKickoffScatter.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffScatter {
    /// Translated from `fBallCoordinateEnd`.
    pub ball_coordinate_end: FieldCoordinate,
    /// Translated from `fScatterDirection`.
    pub scatter_direction: Direction,
    /// Translated from `fRollScatterDirection`.
    pub roll_scatter_direction: i32,
    /// Translated from `fRollScatterDistance`.
    pub roll_scatter_distance: i32,
}

impl ReportKickoffScatter {
    pub fn new(
        ball_coordinate_end: FieldCoordinate,
        scatter_direction: Direction,
        roll_scatter_direction: i32,
        roll_scatter_distance: i32,
    ) -> Self {
        Self {
            ball_coordinate_end,
            scatter_direction,
            roll_scatter_direction,
            roll_scatter_distance,
        }
    }

    pub fn get_ball_coordinate_end(&self) -> &FieldCoordinate {
        &self.ball_coordinate_end
    }

    pub fn get_scatter_direction(&self) -> Direction {
        self.scatter_direction
    }

    pub fn get_roll_scatter_direction(&self) -> i32 {
        self.roll_scatter_direction
    }

    pub fn get_roll_scatter_distance(&self) -> i32 {
        self.roll_scatter_distance
    }
}

impl IReport for ReportKickoffScatter {
    fn get_id(&self) -> ReportId {
        ReportId::KICKOFF_SCATTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffScatter {
        ReportKickoffScatter::new(
            FieldCoordinate::new(5, 7),
            Direction::North,
            3,
            4,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_SCATTER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffScatter");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_scatter_direction(), Direction::North);
        assert_eq!(r.get_roll_scatter_direction(), 3);
        assert_eq!(r.get_roll_scatter_distance(), 4);
    }

    #[test]
    fn ball_coordinate_end() {
        let r = make();
        assert_eq!(r.get_ball_coordinate_end(), &FieldCoordinate::new(5, 7));
    }

    #[test]
    fn different_direction() {
        let r = ReportKickoffScatter::new(FieldCoordinate::new(0, 0), Direction::South, 1, 6);
        assert_eq!(r.get_scatter_direction(), Direction::South);
        assert_eq!(r.get_roll_scatter_direction(), 1);
        assert_eq!(r.get_roll_scatter_distance(), 6);
    }
}
