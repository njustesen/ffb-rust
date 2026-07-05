use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportThrowIn.java`.
#[derive(Debug, Clone)]
pub struct ReportThrowIn {
    pub direction: Direction,
    pub direction_roll: i32,
    pub distance_roll: Vec<i32>,
}

impl ReportThrowIn {
    pub fn new(direction: Direction, direction_roll: i32, distance_roll: Vec<i32>) -> Self {
        Self { direction, direction_roll, distance_roll }
    }

    pub fn get_direction(&self) -> Direction { self.direction }
    pub fn get_direction_roll(&self) -> i32 { self.direction_roll }
    pub fn get_distance_roll(&self) -> &[i32] { &self.distance_roll }
}

impl IReport for ReportThrowIn {
    fn get_id(&self) -> ReportId { ReportId::THROW_IN }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportThrowIn {
        ReportThrowIn::new(Direction::North, 3, vec![2, 4])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::THROW_IN);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "throwIn");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_direction(), Direction::North);
        assert_eq!(r.get_direction_roll(), 3);
        assert_eq!(r.get_distance_roll(), &[2, 4]);
    }
}
