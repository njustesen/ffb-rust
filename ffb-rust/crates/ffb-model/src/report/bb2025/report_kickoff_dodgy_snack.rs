use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffDodgySnack.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffDodgySnack {
    pub roll_home: i32,
    pub roll_away: i32,
    pub player_ids: Vec<String>,
}

impl ReportKickoffDodgySnack {
    pub fn new(roll_home: i32, roll_away: i32, player_ids: Vec<String>) -> Self {
        Self { roll_home, roll_away, player_ids }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "rollHome": self.roll_home,
            "rollAway": self.roll_away,
            "playerIds": self.player_ids,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            player_ids: json["playerIds"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }
    }
}

impl IReport for ReportKickoffDodgySnack {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_DODGY_SNACK }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffDodgySnack {
        ReportKickoffDodgySnack::new(3, 4, vec!["p1".into()])
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::KICKOFF_DODGY_SNACK);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "kickoffDodgySnack");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 3);
        assert_eq!(r.get_player_ids().len(), 1);
    }

    #[test]
    fn roll_away_and_player_id_content() {
        let r = make();
        assert_eq!(r.get_roll_away(), 4);
        assert_eq!(r.get_player_ids()[0], "p1");
    }

    #[test]
    fn multiple_player_ids() {
        let r = ReportKickoffDodgySnack::new(2, 5, vec!["p1".into(), "p2".into()]);
        assert_eq!(r.get_player_ids().len(), 2);
        assert_eq!(r.get_roll_away(), 5);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffDodgySnack::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.player_ids, original.player_ids);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffDodgySnack"));
    }
}
