use crate::enums::ReRollSource;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportReRoll {
    pub player_id: Option<String>,
    pub re_roll_source: ReRollSource,
    pub successful: bool,
    pub roll: i32,
}

impl ReportReRoll {
    pub fn new(
        player_id: Option<String>,
        re_roll_source: ReRollSource,
        successful: bool,
        roll: i32,
    ) -> Self {
        Self { player_id, re_roll_source, successful, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_re_roll_source(&self) -> &ReRollSource { &self.re_roll_source }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportReRoll {
    fn get_id(&self) -> ReportId { ReportId::RE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportReRoll {
        ReportReRoll::new(Some("p1".into()), ReRollSource::new("teamReRoll"), true, 4)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RE_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "reRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert!(r.is_successful());
        assert_eq!(r.get_roll(), 4);
    }

    #[test]
    fn no_player_id() {
        let r = ReportReRoll::new(None, ReRollSource::new("teamReRoll"), true, 3);
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn unsuccessful_roll() {
        let r = ReportReRoll::new(Some("p2".into()), ReRollSource::new("teamReRoll"), false, 2);
        assert!(!r.is_successful());
        assert_eq!(r.get_roll(), 2);
    }
}
