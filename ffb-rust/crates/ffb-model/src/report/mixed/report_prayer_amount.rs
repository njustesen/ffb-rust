use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPrayerAmount.java`.
#[derive(Debug, Clone)]
pub struct ReportPrayerAmount {
    pub tv_home: i32,
    pub tv_away: i32,
    pub prayer_amount: i32,
    pub home_team_receives_prayers: bool,
}

impl ReportPrayerAmount {
    pub fn new(tv_home: i32, tv_away: i32, prayer_amount: i32, home_team_receives_prayers: bool) -> Self {
        Self { tv_home, tv_away, prayer_amount, home_team_receives_prayers }
    }

    pub fn get_tv_home(&self) -> i32 { self.tv_home }
    pub fn get_tv_away(&self) -> i32 { self.tv_away }
    pub fn get_prayer_amount(&self) -> i32 { self.prayer_amount }
    pub fn is_home_team_receives_prayers(&self) -> bool { self.home_team_receives_prayers }
}

impl IReport for ReportPrayerAmount {
    fn get_id(&self) -> ReportId { ReportId::PRAYER_AMOUNT }
}

impl ReportPrayerAmount {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "teamValue": self.tv_home,
            "opponentTeamValue": self.tv_away,
            "homeTeam": self.home_team_receives_prayers,
            "number": self.prayer_amount,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            tv_home: json["teamValue"].as_i64().unwrap_or(0) as i32,
            tv_away: json["opponentTeamValue"].as_i64().unwrap_or(0) as i32,
            prayer_amount: json["number"].as_i64().unwrap_or(0) as i32,
            home_team_receives_prayers: json["homeTeam"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPrayerAmount {
        ReportPrayerAmount::new(1000, 800, 3, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::PRAYER_AMOUNT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "prayerAmount"); }

    #[test]
    fn get_prayer_amount() { assert_eq!(make().get_prayer_amount(), 3); }

    #[test]
    fn is_home_team_receives_prayers() { assert!(make().is_home_team_receives_prayers()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPrayerAmount::from_json(&json);
        assert_eq!(restored.tv_home, original.tv_home);
        assert_eq!(restored.tv_away, original.tv_away);
        assert_eq!(restored.prayer_amount, original.prayer_amount);
        assert_eq!(restored.home_team_receives_prayers, original.home_team_receives_prayers);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("prayerAmount"));
    }
}
