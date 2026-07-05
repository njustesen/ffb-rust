use crate::model::pushback_mode::PushbackMode;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPushback.java`.
#[derive(Debug, Clone)]
pub struct ReportPushback {
    pub defender_id: String,
    pub pushback_mode: PushbackMode,
}

impl ReportPushback {
    pub fn new(defender_id: String, pushback_mode: PushbackMode) -> Self {
        Self { defender_id, pushback_mode }
    }

    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn get_pushback_mode(&self) -> PushbackMode { self.pushback_mode }
}

impl IReport for ReportPushback {
    fn get_id(&self) -> ReportId { ReportId::PUSHBACK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPushback {
        ReportPushback::new("def1".into(), PushbackMode::REGULAR)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PUSHBACK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "pushback");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_defender_id(), "def1");
        assert_eq!(r.get_pushback_mode(), PushbackMode::REGULAR);
    }
}
