use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportStandUpRoll.java`.
/// Note: Java's `ReportStandUpRoll` does NOT extend `ReportSkillRoll`; it is a standalone class.
#[derive(Debug, Clone)]
pub struct ReportStandUpRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub roll: i32,
    pub modifier: i32,
    pub re_rolled: bool,
}

impl ReportStandUpRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        modifier: i32,
        re_rolled: bool,
    ) -> Self {
        Self { player_id, successful, roll, modifier, re_rolled }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_minimum_roll(&self) -> i32 { (4 - self.modifier).max(2) }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }
}

impl IReport for ReportStandUpRoll {
    fn get_id(&self) -> ReportId { ReportId::STAND_UP_ROLL }
}

impl ReportStandUpRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "successful": self.successful,
            "roll": self.roll,
            "modifier": self.modifier,
            "reRolled": self.re_rolled,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            successful: json["successful"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
            modifier: json["modifier"].as_i64().unwrap_or(0) as i32,
            re_rolled: json["reRolled"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportStandUpRoll {
        ReportStandUpRoll::new(Some("p1".into()), true, 4, 1, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::STAND_UP_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "standUpRoll");
    }

    #[test]
    fn minimum_roll_clamped() {
        // modifier=1 → 4-1=3; modifier=3 → 4-3=1 clamped to 2
        assert_eq!(make().get_minimum_roll(), 3);
        let r = ReportStandUpRoll::new(Some("p1".into()), true, 2, 3, false);
        assert_eq!(r.get_minimum_roll(), 2);
    }

    #[test]
    fn get_player_id() {
        assert_eq!(make().get_player_id(), Some("p1"));
        let r = ReportStandUpRoll::new(None, false, 1, 0, false);
        assert_eq!(r.get_player_id(), None);
    }

    #[test]
    fn is_re_rolled() {
        assert!(!make().is_re_rolled());
        let r = ReportStandUpRoll::new(Some("p1".into()), true, 4, 1, true);
        assert!(r.is_re_rolled());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportStandUpRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.modifier, original.modifier);
        assert_eq!(restored.re_rolled, original.re_rolled);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("standUpRoll"));
    }
}
