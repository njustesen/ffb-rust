use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPassBlock.java`.
#[derive(Debug, Clone)]
pub struct ReportPassBlock {
    /// Translated from `fTeamId`.
    pub team_id: String,
    /// Translated from `fPassBlockAvailable`.
    pub pass_block_available: bool,
}

impl ReportPassBlock {
    pub fn new(team_id: String, pass_block_available: bool) -> Self {
        Self { team_id, pass_block_available }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn is_pass_block_available(&self) -> bool {
        self.pass_block_available
    }
}

impl IReport for ReportPassBlock {
    fn get_id(&self) -> ReportId {
        ReportId::PASS_BLOCK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassBlock {
        ReportPassBlock::new("team1".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PASS_BLOCK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "passBlock");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_pass_block_available());
    }
}
