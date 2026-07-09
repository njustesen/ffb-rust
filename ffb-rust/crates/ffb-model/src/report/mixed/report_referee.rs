use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReferee.java`.
#[derive(Debug, Clone)]
pub struct ReportReferee {
    /// `fFoulingPlayerBanned`
    pub fouling_player_banned: bool,
    pub under_scrutiny: bool,
}

impl ReportReferee {
    pub fn new(fouling_player_banned: bool, under_scrutiny: bool) -> Self {
        Self { fouling_player_banned, under_scrutiny }
    }

    pub fn is_fouling_player_banned(&self) -> bool { self.fouling_player_banned }
    pub fn is_under_scrutiny(&self) -> bool { self.under_scrutiny }
}

impl IReport for ReportReferee {
    fn get_id(&self) -> ReportId { ReportId::REFEREE }
}

impl ReportReferee {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "foulingPlayerBanned": self.fouling_player_banned,
            "underScrutiny": self.under_scrutiny,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            fouling_player_banned: json["foulingPlayerBanned"].as_bool().unwrap_or(false),
            under_scrutiny: json["underScrutiny"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportReferee {
        ReportReferee::new(true, false)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::REFEREE); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "referee"); }

    #[test]
    fn is_fouling_player_banned() { assert!(make().is_fouling_player_banned()); }

    #[test]
    fn is_under_scrutiny() { assert!(!make().is_under_scrutiny()); }
    #[test]
    fn get_name_is_nonempty() {
        assert!(!make().get_name().is_empty());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportReferee::from_json(&json);
        assert_eq!(restored.fouling_player_banned, original.fouling_player_banned);
        assert_eq!(restored.under_scrutiny, original.under_scrutiny);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("referee"));
    }
}
