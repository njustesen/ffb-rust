use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportReferee.java`.
#[derive(Debug, Clone)]
pub struct ReportReferee {
    pub fouling_player_banned: bool,
}

impl ReportReferee {
    pub fn new(fouling_player_banned: bool) -> Self {
        Self { fouling_player_banned }
    }

    pub fn is_fouling_player_banned(&self) -> bool { self.fouling_player_banned }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "foulingPlayerBanned": self.fouling_player_banned,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            fouling_player_banned: json["foulingPlayerBanned"].as_bool().unwrap_or(false),
        }
    }
}

impl IReport for ReportReferee {
    fn get_id(&self) -> ReportId { ReportId::REFEREE }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id() {
        assert_eq!(ReportReferee::new(true).get_id(), ReportId::REFEREE);
    }

    #[test]
    fn get_name() {
        assert_eq!(ReportReferee::new(false).get_name(), "referee");
    }

    #[test]
    fn fields() {
        assert!(ReportReferee::new(true).is_fouling_player_banned());
        assert!(!ReportReferee::new(false).is_fouling_player_banned());
    }

    #[test]
    fn banned_true_id_correct() {
        let r = ReportReferee::new(true);
        assert!(r.is_fouling_player_banned());
        assert_eq!(r.get_id(), ReportId::REFEREE);
    }

    #[test]
    fn not_banned_name_correct() {
        let r = ReportReferee::new(false);
        assert!(!r.is_fouling_player_banned());
        assert_eq!(r.get_name(), "referee");
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportReferee::new(true);
        let json = original.to_json_value();
        let restored = ReportReferee::from_json(&json);
        assert_eq!(restored.fouling_player_banned, original.fouling_player_banned);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = ReportReferee::new(true).to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("referee"));
    }
}
