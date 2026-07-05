use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;

/// 1:1 translation of `ReportPassDeviate.java`.
#[derive(Debug, Clone)]
pub struct ReportPassDeviate {
    /// Translated from `fBallCoordinateEnd`.
    pub ball_coordinate_end: FieldCoordinate,
    /// Translated from `fScatterDirection`.
    pub scatter_direction: Direction,
    /// Translated from `fRollScatterDirection`.
    pub roll_scatter_direction: i32,
    /// Translated from `fRollScatterDistance`.
    pub roll_scatter_distance: i32,
    /// Translated from `ttm` (throw-team-mate flag).
    pub ttm: bool,
}

impl ReportPassDeviate {
    pub fn new(
        ball_coordinate_end: FieldCoordinate,
        scatter_direction: Direction,
        roll_scatter_direction: i32,
        roll_scatter_distance: i32,
        ttm: bool,
    ) -> Self {
        Self {
            ball_coordinate_end,
            scatter_direction,
            roll_scatter_direction,
            roll_scatter_distance,
            ttm,
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

    pub fn is_ttm(&self) -> bool {
        self.ttm
    }
}

impl IReport for ReportPassDeviate {
    fn get_id(&self) -> ReportId {
        ReportId::PASS_DEVIATE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassDeviate {
        ReportPassDeviate::new(
            FieldCoordinate::new(10, 5),
            Direction::East,
            6,
            3,
            false,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PASS_DEVIATE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "passDeviate");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_scatter_direction(), Direction::East);
        assert_eq!(r.get_roll_scatter_direction(), 6);
        assert_eq!(r.get_roll_scatter_distance(), 3);
        assert!(!r.is_ttm());
    }
}
