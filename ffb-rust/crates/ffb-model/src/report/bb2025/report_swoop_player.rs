use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopPlayer.java` (bb2025).
#[derive(Debug, Clone)]
pub struct ReportSwoopPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub direction: Direction,
    pub distance: i32,
    pub out_of_bounds: bool,
}

impl ReportSwoopPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        direction: Direction,
        distance: i32,
        out_of_bounds: bool,
    ) -> Self {
        Self { start_coordinate, end_coordinate, direction, distance, out_of_bounds }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_direction(&self) -> Direction { self.direction }
    pub fn get_distance(&self) -> i32 { self.distance }
    pub fn is_out_of_bounds(&self) -> bool { self.out_of_bounds }
}

impl IReport for ReportSwoopPlayer {
    fn get_id(&self) -> ReportId { ReportId::SWOOP_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSwoopPlayer {
        ReportSwoopPlayer::new(
            FieldCoordinate::new(3, 5),
            FieldCoordinate::new(6, 5),
            Direction::East,
            3,
            false,
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SWOOP_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "swoopPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Direction::East);
        assert_eq!(r.get_distance(), 3);
        assert!(!r.is_out_of_bounds());
    }

    #[test]
    fn start_and_end_coordinates() {
        let r = make();
        assert_eq!(r.get_start_coordinate().x, 3);
        assert_eq!(r.get_end_coordinate().x, 6);
    }

    #[test]
    fn out_of_bounds_different_direction() {
        let r = ReportSwoopPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(0, 3),
            Direction::North,
            3,
            true,
        );
        assert_eq!(r.get_direction(), Direction::North);
        assert!(r.is_out_of_bounds());
    }
}
