use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBlockRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportBlockRoll {
    pub choosing_team_id: String,
    pub block_roll: Vec<i32>,
    /// Nullable in Java — `None` when not set.
    pub defender_id: Option<String>,
}

impl ReportBlockRoll {
    pub fn new(
        choosing_team_id: String,
        block_roll: Vec<i32>,
        defender_id: Option<String>,
    ) -> Self {
        Self { choosing_team_id, block_roll, defender_id }
    }

    pub fn get_choosing_team_id(&self) -> &str { &self.choosing_team_id }
    pub fn get_block_roll(&self) -> &[i32] { &self.block_roll }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "choosingTeamId": self.choosing_team_id,
            "blockRoll": self.block_roll,
            "defenderId": self.defender_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            choosing_team_id: json["choosingTeamId"].as_str().unwrap_or("").to_string(),
            block_roll: json["blockRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            defender_id: json["defenderId"].as_str().map(str::to_string),
        }
    }
}

impl IReport for ReportBlockRoll {
    fn get_id(&self) -> ReportId { ReportId::BLOCK_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBlockRoll {
        ReportBlockRoll::new("team1".into(), vec![2, 4, 6], Some("def1".into()))
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BLOCK_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "blockRoll");
    }

    #[test]
    fn get_choosing_team_id() {
        assert_eq!(make().get_choosing_team_id(), "team1");
    }

    #[test]
    fn get_block_roll_and_defender_id() {
        let r = make();
        assert_eq!(r.get_block_roll(), &[2, 4, 6]);
        assert_eq!(r.get_defender_id(), Some("def1"));
    }

    #[test]
    fn no_defender_id() {
        let r = ReportBlockRoll::new("team2".into(), vec![1], None);
        assert_eq!(r.get_defender_id(), None);
        assert_eq!(r.get_choosing_team_id(), "team2");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportBlockRoll::from_json(&json);
        assert_eq!(restored.choosing_team_id, original.choosing_team_id);
        assert_eq!(restored.block_roll, original.block_roll);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("blockRoll"));
    }
}
