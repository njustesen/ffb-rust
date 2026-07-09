use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffPitchInvasion.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffPitchInvasion {
    pub rolls_home: Vec<i32>,
    pub players_affected_home: Vec<bool>,
    pub rolls_away: Vec<i32>,
    pub players_affected_away: Vec<bool>,
}

impl ReportKickoffPitchInvasion {
    pub fn new(
        rolls_home: Vec<i32>,
        players_affected_home: Vec<bool>,
        rolls_away: Vec<i32>,
        players_affected_away: Vec<bool>,
    ) -> Self {
        Self { rolls_home, players_affected_home, rolls_away, players_affected_away }
    }

    pub fn get_rolls_home(&self) -> &[i32] { &self.rolls_home }
    pub fn get_players_affected_home(&self) -> &[bool] { &self.players_affected_home }
    pub fn get_rolls_away(&self) -> &[i32] { &self.rolls_away }
    pub fn get_players_affected_away(&self) -> &[bool] { &self.players_affected_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "rollsHome": self.rolls_home,
            "playersAffectedHome": self.players_affected_home,
            "rollsAway": self.rolls_away,
            "playersAffectedAway": self.players_affected_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            rolls_home: json["rollsHome"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            players_affected_home: json["playersAffectedHome"].as_array().map(|a| a.iter().map(|v| v.as_bool().unwrap_or(false)).collect()).unwrap_or_default(),
            rolls_away: json["rollsAway"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            players_affected_away: json["playersAffectedAway"].as_array().map(|a| a.iter().map(|v| v.as_bool().unwrap_or(false)).collect()).unwrap_or_default(),
        }
    }
}

impl IReport for ReportKickoffPitchInvasion {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_PITCH_INVASION }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffPitchInvasion {
        ReportKickoffPitchInvasion::new(vec![4, 3], vec![true, false], vec![2, 5], vec![false, true])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_PITCH_INVASION);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffPitchInvasion");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_rolls_home(), &[4, 3]);
        assert_eq!(r.get_players_affected_home(), &[true, false]);
        assert_eq!(r.get_rolls_away(), &[2, 5]);
    }

    #[test]
    fn players_affected_away() {
        let r = make();
        assert_eq!(r.get_players_affected_away(), &[false, true]);
    }

    #[test]
    fn empty_vectors() {
        let r = ReportKickoffPitchInvasion::new(vec![], vec![], vec![], vec![]);
        assert!(r.get_rolls_home().is_empty());
        assert!(r.get_players_affected_away().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffPitchInvasion::from_json(&json);
        assert_eq!(restored.rolls_home, original.rolls_home);
        assert_eq!(restored.players_affected_home, original.players_affected_home);
        assert_eq!(restored.rolls_away, original.rolls_away);
        assert_eq!(restored.players_affected_away, original.players_affected_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffPitchInvasion"));
    }
}
