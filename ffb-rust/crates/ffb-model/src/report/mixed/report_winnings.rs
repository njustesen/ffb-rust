use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWinnings.java`.
#[derive(Debug, Clone)]
pub struct ReportWinnings {
    pub winnings_home: i32,
    pub winnings_away: i32,
}

impl ReportWinnings {
    pub fn new(winnings_home: i32, winnings_away: i32) -> Self {
        Self { winnings_home, winnings_away }
    }

    pub fn get_winnings_home(&self) -> i32 { self.winnings_home }
    pub fn get_winnings_away(&self) -> i32 { self.winnings_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "winningsHome": self.winnings_home,
            "winningsAway": self.winnings_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            winnings_home: json["winningsHome"].as_i64().unwrap_or(0) as i32,
            winnings_away: json["winningsAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportWinnings {
    fn get_id(&self) -> ReportId { ReportId::WINNINGS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWinnings {
        ReportWinnings::new(50000, 30000)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::WINNINGS); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "winnings"); }

    #[test]
    fn get_winnings_home() { assert_eq!(make().get_winnings_home(), 50000); }

    #[test]
    fn get_winnings_away() { assert_eq!(make().get_winnings_away(), 30000); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportWinnings::from_json(&json);
        assert_eq!(restored.winnings_home, original.winnings_home);
        assert_eq!(restored.winnings_away, original.winnings_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("winnings"));
    }
}
