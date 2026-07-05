use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPlaceBallDirection.java`.
#[derive(Debug, Clone)]
pub struct ReportPlaceBallDirection {
    pub player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportPlaceBallDirection {
    pub fn new(player_id: Option<String>, direction: Option<Direction>) -> Self {
        Self { player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportPlaceBallDirection {
    fn get_id(&self) -> ReportId { ReportId::PLACE_BALL_DIRECTION }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPlaceBallDirection {
        ReportPlaceBallDirection::new(Some("p1".into()), Some(Direction::North))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PLACE_BALL_DIRECTION); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "placedBallDirection"); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::North)); }
}
