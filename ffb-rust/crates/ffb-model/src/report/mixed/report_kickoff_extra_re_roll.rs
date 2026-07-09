use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffExtraReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffExtraReRoll {
    pub roll_home: i32,
    pub roll_away: i32,
    pub team_id: Option<String>,
}

impl ReportKickoffExtraReRoll {
    pub fn new(roll_home: i32, roll_away: i32, team_id: Option<String>) -> Self {
        Self { roll_home, roll_away, team_id }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
}

impl IReport for ReportKickoffExtraReRoll {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_EXTRA_RE_ROLL }
}

impl ReportKickoffExtraReRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            team_id: json["teamId"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffExtraReRoll {
        ReportKickoffExtraReRoll::new(4, 2, Some("team1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_EXTRA_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "extraReRoll"); }

    #[test]
    fn get_roll_home() { assert_eq!(make().get_roll_home(), 4); }

    #[test]
    fn get_roll_away() { assert_eq!(make().get_roll_away(), 2); }

    #[test]
    fn get_team_id_none() {
        let r = ReportKickoffExtraReRoll::new(1, 1, None);
        assert!(r.get_team_id().is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffExtraReRoll::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.team_id, original.team_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("extraReRoll"));
    }
}
