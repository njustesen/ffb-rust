use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPassBlock.java`.
#[derive(Debug, Clone)]
pub struct ReportPassBlock {
    /// Translated from `fTeamId`.
    pub team_id: String,
    /// Translated from `fPassBlockAvailable`.
    pub pass_block_available: bool,
}

impl ReportPassBlock {
    pub fn new(team_id: String, pass_block_available: bool) -> Self {
        Self { team_id, pass_block_available }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn is_pass_block_available(&self) -> bool {
        self.pass_block_available
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "passBlockAvailable": self.pass_block_available,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            pass_block_available: json["passBlockAvailable"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportPassBlock {
    fn get_id(&self) -> ReportId {
        ReportId::PASS_BLOCK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPassBlock {
        ReportPassBlock::new("team1".into(), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PASS_BLOCK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "passBlock");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert!(r.is_pass_block_available());
    }

    #[test]
    fn pass_block_unavailable() {
        let r = ReportPassBlock::new("team2".into(), false);
        assert!(!r.is_pass_block_available());
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn different_team_id() {
        let r = ReportPassBlock::new("teamX".into(), true);
        assert_eq!(r.get_team_id(), "teamX");
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPassBlock::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.pass_block_available, original.pass_block_available);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("passBlock"));
    }
}
