use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportKickoffSequenceActivationsCount.java`.
#[derive(Debug, Clone)]
pub struct ReportKickoffSequenceActivationsCount {
    pub amount: i32,
    pub available: i32,
    pub limit: i32,
}

impl ReportKickoffSequenceActivationsCount {
    pub fn new(amount: i32, available: i32, limit: i32) -> Self {
        Self { amount, available, limit }
    }

    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn get_available(&self) -> i32 { self.available }
    pub fn get_limit(&self) -> i32 { self.limit }
}

impl IReport for ReportKickoffSequenceActivationsCount {
    fn get_id(&self) -> ReportId { ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT }
}

impl ReportKickoffSequenceActivationsCount {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "nrOfPlayers": self.amount,
            "number": self.available,
            "nrOfPlayersAllowed": self.limit,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            amount: json["nrOfPlayers"].as_i64().unwrap_or(0) as i32,
            available: json["number"].as_i64().unwrap_or(0) as i32,
            limit: json["nrOfPlayersAllowed"].as_i64().unwrap_or(0) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportKickoffSequenceActivationsCount {
        ReportKickoffSequenceActivationsCount::new(2, 5, 3)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "kickoffSequenceActivationsCount"); }

    #[test]
    fn get_limit() { assert_eq!(make().get_limit(), 3); }

    #[test]
    fn get_amount() { assert_eq!(make().get_amount(), 2); }

    #[test]
    fn get_available() { assert_eq!(make().get_available(), 5); }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportKickoffSequenceActivationsCount::from_json(&json);
        assert_eq!(restored.amount, original.amount);
        assert_eq!(restored.available, original.available);
        assert_eq!(restored.limit, original.limit);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("kickoffSequenceActivationsCount"));
    }
}
