use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlockRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBlockRoll {
    pub choosing_team_id: String,
    pub block_roll: Vec<i32>,
    /// Nullable in Java — `None` when not set.
    pub defender_id: Option<String>,
}

impl ReportBlockRoll {
    pub fn new(
        choosing_team_id: String,
        block_roll: Vec<i32>,
        defender_id: Option<String>,
    ) -> Self {
        Self { choosing_team_id, block_roll, defender_id }
    }

    pub fn get_choosing_team_id(&self) -> &str { &self.choosing_team_id }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportBlockRoll {
    fn get_id(&self) -> ReportId { ReportId::BLOCK_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlockRoll {
        ReportBlockRoll::new("team1".into(), vec![2, 4, 6], Some("def1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BLOCK_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "blockRoll");
    }

    #[test]
    fn get_choosing_team_id() {
        assert_eq!(make().get_choosing_team_id(), "team1");
    }
}
