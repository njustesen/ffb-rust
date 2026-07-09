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

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "reRollSource": self.re_roll_source.name,
            "successful": self.successful,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            re_roll_source: ReRollSource::new(json["reRollSource"].as_str().unwrap_or("")),
            successful: json["successful"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportReRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.re_roll_source.name, original.re_roll_source.name);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("reRoll"));
    }
}
