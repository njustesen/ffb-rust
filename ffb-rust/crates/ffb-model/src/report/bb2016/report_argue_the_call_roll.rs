use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportArgueTheCallRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportArgueTheCallRoll {
    pub player_id: String,
    pub successful: bool,
    pub coach_banned: bool,
    pub roll: i32,
}

impl ReportArgueTheCallRoll {
    pub fn new(player_id: String, successful: bool, coach_banned: bool, roll: i32) -> Self {
        Self { player_id, successful, coach_banned, roll }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_coach_banned(&self) -> bool { self.coach_banned }
    pub fn get_roll(&self) -> i32 { self.roll }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "successful": self.successful,
            "coachBanned": self.coach_banned,
            "roll": self.roll,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().unwrap_or("").to_string(),
            successful: json["successful"].as_bool().unwrap_or(false),
            coach_banned: json["coachBanned"].as_bool().unwrap_or(false),
            roll: json["roll"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportArgueTheCallRoll {
    fn get_id(&self) -> ReportId { ReportId::ARGUE_THE_CALL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportArgueTheCallRoll {
        ReportArgueTheCallRoll::new("p1".into(), true, false, 5)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::ARGUE_THE_CALL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "argueTheCall");
    }

    #[test]
    fn fields() {
        let r = make();
        assert!(r.is_successful());
        assert!(!r.is_coach_banned());
        assert_eq!(r.get_roll(), 5);
    }

    #[test]
    fn coach_banned_and_player_id() {
        let r = ReportArgueTheCallRoll::new("coach1".into(), false, true, 2);
        assert!(!r.is_successful());
        assert!(r.is_coach_banned());
        assert_eq!(r.get_player_id(), "coach1");
    }

    #[test]
    fn unsuccessful_stores_roll() {
        let r = ReportArgueTheCallRoll::new("p2".into(), false, false, 1);
        assert_eq!(r.get_roll(), 1);
        assert!(!r.is_successful());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportArgueTheCallRoll::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.successful, original.successful);
        assert_eq!(restored.coach_banned, original.coach_banned);
        assert_eq!(restored.roll, original.roll);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("argueTheCall"));
    }
}
