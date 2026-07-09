use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportInducementsBought.java`.
#[derive(Debug, Clone)]
pub struct ReportInducementsBought {
    pub team_id: String,
    pub nr_of_inducements: i32,
    pub nr_of_stars: i32,
    pub nr_of_mercenaries: i32,
    pub gold: i32,
}

impl ReportInducementsBought {
    pub fn new(
        team_id: String,
        nr_of_inducements: i32,
        nr_of_stars: i32,
        nr_of_mercenaries: i32,
        gold: i32,
    ) -> Self {
        Self { team_id, nr_of_inducements, nr_of_stars, nr_of_mercenaries, gold }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_nr_of_inducements(&self) -> i32 { self.nr_of_inducements }
    pub fn get_nr_of_stars(&self) -> i32 { self.nr_of_stars }
    pub fn get_nr_of_mercenaries(&self) -> i32 { self.nr_of_mercenaries }
    pub fn get_gold(&self) -> i32 { self.gold }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamId": self.team_id,
            "nrOfInducements": self.nr_of_inducements,
            "nrOfStars": self.nr_of_stars,
            "nrOfMercenaries": self.nr_of_mercenaries,
            "gold": self.gold,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            team_id: json["teamId"].as_str().unwrap_or("").to_string(),
            nr_of_inducements: json["nrOfInducements"].as_i64().unwrap_or(0) as i32,
            nr_of_stars: json["nrOfStars"].as_i64().unwrap_or(0) as i32,
            nr_of_mercenaries: json["nrOfMercenaries"].as_i64().unwrap_or(0) as i32,
            gold: json["gold"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportInducementsBought {
    fn get_id(&self) -> ReportId { ReportId::INDUCEMENTS_BOUGHT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInducementsBought {
        ReportInducementsBought::new("team1".into(), 3, 1, 0, 150000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INDUCEMENTS_BOUGHT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "inducementsBought");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_nr_of_inducements(), 3);
        assert_eq!(r.get_nr_of_stars(), 1);
        assert_eq!(r.get_gold(), 150000);
    }

    #[test]
    fn mercenaries_count() {
        let r = ReportInducementsBought::new("team2".into(), 2, 0, 1, 80000);
        assert_eq!(r.get_nr_of_mercenaries(), 1);
        assert_eq!(r.get_nr_of_stars(), 0);
    }

    #[test]
    fn zero_inducements() {
        let r = ReportInducementsBought::new("team3".into(), 0, 0, 0, 0);
        assert_eq!(r.get_nr_of_inducements(), 0);
        assert_eq!(r.get_gold(), 0);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportInducementsBought::from_json(&json);
        assert_eq!(restored.team_id, original.team_id);
        assert_eq!(restored.nr_of_inducements, original.nr_of_inducements);
        assert_eq!(restored.nr_of_stars, original.nr_of_stars);
        assert_eq!(restored.nr_of_mercenaries, original.nr_of_mercenaries);
        assert_eq!(restored.gold, original.gold);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("inducementsBought"));
    }
}
