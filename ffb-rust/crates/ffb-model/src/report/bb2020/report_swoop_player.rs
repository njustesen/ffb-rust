use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopPlayer.java` (bb2020).
#[derive(Debug, Clone)]
pub struct ReportSwoopPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub direction: Direction,
    pub distance: i32,
}

impl ReportSwoopPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        direction: Direction,
        distance: i32,
    ) -> Self {
        Self { start_coordinate, end_coordinate, direction, distance }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_direction(&self) -> Direction { self.direction }
    pub fn get_distance(&self) -> i32 { self.distance }
}

impl IReport for ReportSwoopPlayer {
    fn get_id(&self) -> ReportId { ReportId::SWOOP_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSwoopPlayer {
        ReportSwoopPlayer::new(
            FieldCoordinate::new(5, 7),
            FieldCoordinate::new(8, 7),
            Direction::East,
            3,
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
    }

    #[test]
    fn start_and_end_coordinates() {
        let r = make();
        assert_eq!(r.get_start_coordinate().x, 5);
        assert_eq!(r.get_end_coordinate().x, 8);
    }

    #[test]
    fn different_direction() {
        let r = ReportSwoopPlayer::new(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(0, 2),
            Direction::North,
            2,
        );
        assert_eq!(r.get_direction(), Direction::North);
        assert_eq!(r.get_distance(), 2);
    }
}
