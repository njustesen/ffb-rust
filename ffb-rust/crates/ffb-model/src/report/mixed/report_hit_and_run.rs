use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportHitAndRun.java`.
#[derive(Debug, Clone)]
pub struct ReportHitAndRun {
    pub player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportHitAndRun {
    pub fn new(player_id: Option<String>, direction: Option<Direction>) -> Self {
        Self { player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportHitAndRun {
    fn get_id(&self) -> ReportId { ReportId::HIT_AND_RUN }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportHitAndRun {
        ReportHitAndRun::new(Some("p1".into()), Some(Direction::North))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::HIT_AND_RUN); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "hitAndRun"); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::North)); }
}
