use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBriberyAndCorruptionReRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBriberyAndCorruptionReRoll {
    pub team_id: Option<String>,
    pub action: String,
}

impl ReportBriberyAndCorruptionReRoll {
    pub fn new(team_id: Option<String>, action: String) -> Self {
        Self { team_id, action }
    }

    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_action(&self) -> &str { &self.action }
}

impl IReport for ReportBriberyAndCorruptionReRoll {
    fn get_id(&self) -> ReportId { ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL }
}

impl ReportBriberyAndCorruptionReRoll {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "briberyAncCorruptionAction": self.action,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().map(str::to_string),
            action: json["briberyAncCorruptionAction"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBriberyAndCorruptionReRoll {
        ReportBriberyAndCorruptionReRoll::new(Some("team1".into()), "USE".into())
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::BRIBERY_AND_CORRUPTION_RE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "briberyAndCorruptionReRoll"); }

    #[test]
    fn get_action() { assert_eq!(make().get_action(), "USE"); }

    #[test]
    fn get_team_id() { assert_eq!(make().get_team_id(), Some("team1")); }

    #[test]
    fn none_team_id() {
        let r = ReportBriberyAndCorruptionReRoll::new(None, "DECLINE".into());
        assert_eq!(r.get_team_id(), None);
        assert_eq!(r.get_action(), "DECLINE");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBriberyAndCorruptionReRoll::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.action, original.action);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("briberyAndCorruptionReRoll"));
    }
}
