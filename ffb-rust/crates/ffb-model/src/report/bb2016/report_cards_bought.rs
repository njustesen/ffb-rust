use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCardsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportCardsBought {
    pub team_id: String,
    pub nr_of_cards: i32,
    pub gold: i32,
}

impl ReportCardsBought {
    pub fn new(team_id: String, nr_of_cards: i32, gold: i32) -> Self {
        Self { team_id, nr_of_cards, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_nr_of_cards(&self) -> i32 { self.nr_of_cards }
    pub fn get_gold(&self) -> i32 { self.gold }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "nrOfCards": self.nr_of_cards,
            "gold": self.gold,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            nr_of_cards: json["nrOfCards"].as_i64().unwrap_or(0) as i32,
            gold: json["gold"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportCardsBought {
    fn get_id(&self) -> ReportId { ReportId::CARDS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCardsBought {
        ReportCardsBought::new("team1".into(), 2, 50000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::CARDS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "cardsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_nr_of_cards(), 2);
        assert_eq!(r.get_gold(), 50000);
    }

    #[test]
    fn zero_cards_zero_gold() {
        let r = ReportCardsBought::new("team2".into(), 0, 0);
        assert_eq!(r.get_nr_of_cards(), 0);
        assert_eq!(r.get_gold(), 0);
    }

    #[test]
    fn different_team_id() {
        let r = ReportCardsBought::new("away_team".into(), 5, 100000);
        assert_eq!(r.get_team_id(), "away_team");
        assert_eq!(r.get_nr_of_cards(), 5);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCardsBought::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.nr_of_cards, original.nr_of_cards);
        assert_eq!(restored.gold, original.gold);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("cardsBought"));
    }
}
