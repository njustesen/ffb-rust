use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportScatterBall.java`.
#[derive(Debug, Clone)]
pub struct ReportScatterBall {
    pub directions: Vec<Direction>,
    pub rolls: Vec<i32>,
    pub gust_of_wind: bool,
}

impl ReportScatterBall {
    pub fn new(directions: Vec<Direction>, rolls: Vec<i32>, gust_of_wind: bool) -> Self {
        Self { directions, rolls, gust_of_wind }
    }

    pub fn get_directions(&self) -> &[Direction] { &self.directions }
    pub fn get_rolls(&self) -> &[i32] { &self.rolls }
    pub fn is_gust_of_wind(&self) -> bool { self.gust_of_wind }
}

impl IReport for ReportScatterBall {
    fn get_id(&self) -> ReportId { ReportId::SCATTER_BALL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportScatterBall {
        ReportScatterBall::new(vec![Direction::North, Direction::East], vec![3, 5], false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SCATTER_BALL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "scatterBall");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_directions(), &[Direction::North, Direction::East]);
        assert_eq!(r.get_rolls(), &[3, 5]);
        assert!(!r.is_gust_of_wind());
    }
}
