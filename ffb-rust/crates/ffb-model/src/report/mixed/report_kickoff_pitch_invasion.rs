use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffPitchInvasion.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffPitchInvasion {
    pub roll_home: i32,
    pub roll_away: i32,
    pub amount: i32,
    pub affected_players: Vec<String>,
}

impl ReportKickoffPitchInvasion {
    pub fn new(
        roll_home: i32,
        roll_away: i32,
        amount: i32,
        affected_players: Vec<String>,
    ) -> Self {
        Self { roll_home, roll_away, amount, affected_players }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_affected_players(&self) -> &[String] { &self.affected_players }
}

impl IReport for ReportKickoffPitchInvasion {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_PITCH_INVASION }
}

impl ReportKickoffPitchInvasion {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
            "amount": self.amount,
            "playerIds": self.affected_players,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            amount: json["amount"].as_i64().unwrap_or(0) as i32,
            affected_players: json["playerIds"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffPitchInvasion {
        ReportKickoffPitchInvasion::new(3, 2, 1, vec!["p1".into()])
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_PITCH_INVASION); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffPitchInvasion"); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 1); }

    #[test]
    fn get_roll_home_and_away() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert_eq!(r.get_roll_away(), 2);
    }

    #[test]
    fn get_affected_players() {
        let r = make();
        assert_eq!(r.get_affected_players(), &["p1".to_string()]);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffPitchInvasion::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.amount, original.amount);
        assert_eq!(restored.affected_players, original.affected_players);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffPitchInvasion"));
    }
}
