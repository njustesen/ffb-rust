use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWinningsRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportWinningsRoll {
    pub winnings_roll_home: i32,
    pub winnings_home: i32,
    pub winnings_roll_away: i32,
    pub winnings_away: i32,
}

impl ReportWinningsRoll {
    pub fn new(
        winnings_roll_home: i32,
        winnings_home: i32,
        winnings_roll_away: i32,
        winnings_away: i32,
    ) -> Self {
        Self { winnings_roll_home, winnings_home, winnings_roll_away, winnings_away }
    }

    pub fn get_winnings_roll_home(&self) -> i32 { self.winnings_roll_home }
    pub fn get_winnings_home(&self) -> i32 { self.winnings_home }
    pub fn get_winnings_roll_away(&self) -> i32 { self.winnings_roll_away }
    pub fn get_winnings_away(&self) -> i32 { self.winnings_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "winningsRollHome": self.winnings_roll_home,
            "winningsHome": self.winnings_home,
            "winningsRollAway": self.winnings_roll_away,
            "winningsAway": self.winnings_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            winnings_roll_home: json["winningsRollHome"].as_i64().unwrap_or(0) as i32,
            winnings_home: json["winningsHome"].as_i64().unwrap_or(0) as i32,
            winnings_roll_away: json["winningsRollAway"].as_i64().unwrap_or(0) as i32,
            winnings_away: json["winningsAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportWinningsRoll {
    fn get_id(&self) -> ReportId { ReportId::WINNINGS_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWinningsRoll {
        ReportWinningsRoll::new(4, 40000, 2, 20000)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::WINNINGS_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "winningsRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_winnings_roll_home(), 4);
        assert_eq!(r.get_winnings_home(), 40000);
        assert_eq!(r.get_winnings_away(), 20000);
    }

    #[test]
    fn roll_away_stored() {
        let r = make();
        assert_eq!(r.get_winnings_roll_away(), 2);
    }

    #[test]
    fn zero_winnings() {
        let r = ReportWinningsRoll::new(0, 0, 0, 0);
        assert_eq!(r.get_winnings_home(), 0);
        assert_eq!(r.get_winnings_away(), 0);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportWinningsRoll::from_json(&json);
        assert_eq!(restored.winnings_roll_home, original.winnings_roll_home);
        assert_eq!(restored.winnings_home, original.winnings_home);
        assert_eq!(restored.winnings_roll_away, original.winnings_roll_away);
        assert_eq!(restored.winnings_away, original.winnings_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("winningsRoll"));
    }
}
