use crate::enums::Direction;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRaidingParty.java`.
#[derive(Debug, Clone)]
pub struct ReportRaidingParty {
    pub player_id: Option<String>,
    pub other_player_id: Option<String>,
    pub direction: Option<Direction>,
}

impl ReportRaidingParty {
    pub fn new(
        player_id: Option<String>,
        other_player_id: Option<String>,
        direction: Option<Direction>,
    ) -> Self {
        Self { player_id, other_player_id, direction }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_other_player_id(&self) -> Option<&str> { self.other_player_id.as_deref() }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

impl IReport for ReportRaidingParty {
    fn get_id(&self) -> ReportId { ReportId::RAIDING_PARTY }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRaidingParty {
        ReportRaidingParty::new(Some("p1".into()), Some("p2".into()), Some(Direction::Northeast))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::RAIDING_PARTY); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "raidingParty"); }

    #[test]
    fn get_other_player_id() { assert_eq!(make().get_other_player_id(), Some("p2")); }

    #[test]
    fn get_direction() { assert_eq!(make().get_direction(), Some(Direction::Northeast)); }
}
