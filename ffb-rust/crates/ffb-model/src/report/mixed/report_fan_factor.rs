use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFanFactor.java`.
#[derive(Debug, Clone)]
pub struct ReportFanFactor {
    pub roll: i32,
    pub dedicated_fans: i32,
    pub result: i32,
    pub team_id: Option<String>,
}

impl ReportFanFactor {
    pub fn new(roll: i32, dedicated_fans: i32, team_id: Option<String>) -> Self {
        let result = roll + dedicated_fans;
        Self { roll, dedicated_fans, result, team_id }
    }

    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_dedicated_fans(&self) -> i32 { self.dedicated_fans }
    pub fn get_result(&self) -> i32 { self.result }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IReport for ReportFanFactor {
    fn get_id(&self) -> ReportId { ReportId::FAN_FACTOR }
}

impl ReportFanFactor {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "dedicatedFans": self.dedicated_fans,
            "dedicatedFansRoll": self.roll,
            "dedicatedFansResult": self.result,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        let roll = json["dedicatedFansRoll"].as_i64().unwrap_or(0) as i32;
        let dedicated_fans = json["dedicatedFans"].as_i64().unwrap_or(0) as i32;
        let team_id = json["teamId"].as_str().map(str::to_string);
        Self {
            roll,
            dedicated_fans,
            result: roll + dedicated_fans,
            team_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFanFactor {
        ReportFanFactor::new(3, 2, Some("team1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::FAN_FACTOR); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "fanFactor"); }

    #[test]
    fn get_result() { assert_eq!(make().get_result(), 5); }

    #[test]
    fn get_roll_and_dedicated_fans() {
        assert_eq!(make().get_roll(), 3);
        assert_eq!(make().get_dedicated_fans(), 2);
    }

    #[test]
    fn get_team_id() { assert_eq!(make().get_team_id(), Some("team1")); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFanFactor::from_json(&json);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.dedicated_fans, original.dedicated_fans);
        assert_eq!(restored.result, original.result);
        assert_eq!(restored.team_id, original.team_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("fanFactor"));
    }
}
