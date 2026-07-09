use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSpectators.java`.
#[derive(Debug, Clone)]
pub struct ReportSpectators {
    pub spectator_roll_home: Vec<i32>,
    pub spectators_home: i32,
    pub fame_home: i32,
    pub spectator_roll_away: Vec<i32>,
    pub spectators_away: i32,
    pub fame_away: i32,
}

impl ReportSpectators {
    pub fn new(
        spectator_roll_home: Vec<i32>,
        spectators_home: i32,
        fame_home: i32,
        spectator_roll_away: Vec<i32>,
        spectators_away: i32,
        fame_away: i32,
    ) -> Self {
        Self { spectator_roll_home, spectators_home, fame_home, spectator_roll_away, spectators_away, fame_away }
    }

    pub fn get_spectator_roll_home(&self) -> &[i32] { &self.spectator_roll_home }
    pub fn get_spectators_home(&self) -> i32 { self.spectators_home }
    pub fn get_fame_home(&self) -> i32 { self.fame_home }
    pub fn get_spectator_roll_away(&self) -> &[i32] { &self.spectator_roll_away }
    pub fn get_spectators_away(&self) -> i32 { self.spectators_away }
    pub fn get_fame_away(&self) -> i32 { self.fame_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "spectatorRollHome": self.spectator_roll_home,
            "spectatorsHome": self.spectators_home,
            "fameHome": self.fame_home,
            "spectatorRollAway": self.spectator_roll_away,
            "spectatorsAway": self.spectators_away,
            "fameAway": self.fame_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            spectator_roll_home: json["spectatorRollHome"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            spectators_home: json["spectatorsHome"].as_i64().unwrap_or(0) as i32,
            fame_home: json["fameHome"].as_i64().unwrap_or(0) as i32,
            spectator_roll_away: json["spectatorRollAway"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            spectators_away: json["spectatorsAway"].as_i64().unwrap_or(0) as i32,
            fame_away: json["fameAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportSpectators {
    fn get_id(&self) -> ReportId { ReportId::SPECTATORS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSpectators {
        ReportSpectators::new(vec![3, 4], 35000, 1, vec![2, 5], 20000, 0)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SPECTATORS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "spectators");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_spectators_home(), 35000);
        assert_eq!(r.get_fame_home(), 1);
        assert_eq!(r.get_spectators_away(), 20000);
    }

    #[test]
    fn rolls_and_fame_away() {
        let r = make();
        assert_eq!(r.get_spectator_roll_home(), &[3, 4]);
        assert_eq!(r.get_spectator_roll_away(), &[2, 5]);
        assert_eq!(r.get_fame_away(), 0);
    }

    #[test]
    fn equal_spectators() {
        let r = ReportSpectators::new(vec![3], 30000, 2, vec![3], 30000, 2);
        assert_eq!(r.get_spectators_home(), r.get_spectators_away());
        assert_eq!(r.get_fame_home(), r.get_fame_away());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSpectators::from_json(&json);
        assert_eq!(restored.spectator_roll_home, original.spectator_roll_home);
        assert_eq!(restored.spectators_home, original.spectators_home);
        assert_eq!(restored.fame_home, original.fame_home);
        assert_eq!(restored.spectator_roll_away, original.spectator_roll_away);
        assert_eq!(restored.spectators_away, original.spectators_away);
        assert_eq!(restored.fame_away, original.fame_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("spectators"));
    }
}
