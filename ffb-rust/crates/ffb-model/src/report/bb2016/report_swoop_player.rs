use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;
use crate::enums::Direction;

/// 1:1 translation of `ReportSwoopPlayer.java` (bb2016).
#[derive(Debug, Clone)]
pub struct ReportSwoopPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
}

impl ReportSwoopPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        directions: Vec<Direction>,
        rolls: Vec<i32>,
    ) -> Self {
        Self { start_coordinate, end_coordinate, directions, rolls }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
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
            vec![Direction::North, Direction::East],
            vec![3, 5],
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
        assert_eq!(r.get_directions().len(), 2);
        assert_eq!(r.get_rolls(), &[3, 5]);
    }

    #[test]
    fn coordinates_stored() {
        let r = make();
        assert_eq!(r.get_start_coordinate().x, 5);
        assert_eq!(r.get_start_coordinate().y, 7);
        assert_eq!(r.get_end_coordinate().x, 8);
        assert_eq!(r.get_end_coordinate().y, 7);
    }

    #[test]
    fn directions_contents() {
        let r = make();
        assert_eq!(r.get_directions()[0], Direction::North);
        assert_eq!(r.get_directions()[1], Direction::East);
    }
}
