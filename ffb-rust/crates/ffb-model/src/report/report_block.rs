use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlock.java`.
#[derive(Debug, Clone)]
pub struct ReportBlock {
    pub defender_id: String,
}

impl ReportBlock {
    pub fn new(defender_id: String) -> Self {
        Self { defender_id }
    }

    pub fn get_defender_id(&self) -> &str { &self.defender_id }
}

impl IReport for ReportBlock {
    fn get_id(&self) -> ReportId { ReportId::BLOCK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlock {
        ReportBlock::new("def1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BLOCK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "block");
    }

    #[test]
    fn get_defender_id() {
        assert_eq!(make().get_defender_id(), "def1");
    }

    #[test]
    fn different_defender_id() {
        let r = ReportBlock::new("def99".into());
        assert_eq!(r.get_defender_id(), "def99");
    }

    #[test]
    fn report_name_is_block() {
        assert_eq!(ReportBlock::new("x".into()).get_name(), "block");
    }
}
