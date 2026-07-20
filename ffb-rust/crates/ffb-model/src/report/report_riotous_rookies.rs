use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRiotousRookies.java`.
#[derive(Debug, Clone)]
pub struct ReportRiotousRookies {
    pub roll: Vec<i32>,
    pub amount: i32,
    pub team_id: String,
}

impl ReportRiotousRookies {
    pub fn new(roll: Vec<i32>, amount: i32, team_id: String) -> Self {
        Self { roll, amount, team_id }
    }

    pub fn get_roll(&self) -> &[i32] { &self.roll }
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_team_id(&self) -> &str { &self.team_id }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "riotousRoll": self.roll,
            "riotousAmount": self.amount,
            "teamId": self.team_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll: json["riotousRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            amount: json["riotousAmount"].as_i64().unwrap_or(0) as i32,
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
        }
    }
}

impl IReport for ReportRiotousRookies {
    fn get_id(&self) -> ReportId { ReportId::RIOTOUS_ROOKIES }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRiotousRookies {
        ReportRiotousRookies::new(vec![2, 3], 1, "team1".into())
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportRiotousRookies::from_json(&json);
        assert_eq!(restored.roll, original.roll);
        assert_eq!(restored.amount, original.amount);
        assert_eq!(restored.team_id, original.team_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("riotousRookies"));
    }
}
