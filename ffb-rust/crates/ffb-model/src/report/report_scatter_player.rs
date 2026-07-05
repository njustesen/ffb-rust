use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::types::FieldCoordinate;

/// 1:1 translation of `ReportScatterPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportScatterPlayer {
    pub start_coordinate: FieldCoordinate,
    pub end_coordinate: FieldCoordinate,
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
    pub scatter: Option<bool>,
}

impl ReportScatterPlayer {
    pub fn new(
        start_coordinate: FieldCoordinate,
        end_coordinate: FieldCoordinate,
        directions: Vec<Direction>,
        rolls: Vec<i32>,
        scatter: Option<bool>,
    ) -> Self {
        Self { start_coordinate, end_coordinate, directions, rolls, scatter }
    }

    pub fn get_start_coordinate(&self) -> &FieldCoordinate { &self.start_coordinate }
    pub fn get_end_coordinate(&self) -> &FieldCoordinate { &self.end_coordinate }
    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
    pub fn get_scatter(&self) -> Option<bool> { self.scatter }
}

impl IReport for ReportScatterPlayer {
    fn get_id(&self) -> ReportId { ReportId::SCATTER_PLAYER }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportScatterPlayer {
        ReportScatterPlayer::new(
            FieldCoordinate::new(3, 5),
            FieldCoordinate::new(4, 5),
            vec![Direction::East],
            vec![3],
            Some(true),
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SCATTER_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "scatterPlayer");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_directions(), &[Direction::East]);
        assert_eq!(r.get_rolls(), &[3]);
        assert_eq!(r.get_scatter(), Some(true));
    }
}
