use crate::enums::ReRollSource;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlockReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBlockReRoll {
    pub block_roll: Vec<i32>,
    pub player_id: Option<String>,
    pub re_roll_source: Option<ReRollSource>,
}

impl ReportBlockReRoll {
    pub fn new(
        block_roll: Vec<i32>,
        player_id: Option<String>,
        re_roll_source: Option<ReRollSource>,
    ) -> Self {
        Self { block_roll, player_id, re_roll_source }
    }

    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_re_roll_source(&self) -> Option<&ReRollSource> { self.re_roll_source.as_ref() }
}

impl IReport for ReportBlockReRoll {
    fn get_id(&self) -> ReportId { ReportId::BLOCK_RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlockReRoll {
        ReportBlockReRoll::new(vec![2, 5], Some("p1".into()), None)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BLOCK_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "blockReRoll"); }

    #[test]
    fn get_block_roll() { assert_eq!(make().get_block_roll(), &[2, 5]); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn re_roll_source_is_none() { assert!(make().get_re_roll_source().is_none()); }
}
