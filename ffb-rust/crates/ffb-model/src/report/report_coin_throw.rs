use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportCoinThrow.java`.
#[derive(Debug, Clone)]
pub struct ReportCoinThrow {
    pub coin_throw_heads: bool,
    pub coach: String,
    pub coin_choice_heads: bool,
}

impl ReportCoinThrow {
    pub fn new(coin_throw_heads: bool, coach: String, coin_choice_heads: bool) -> Self {
        Self { coin_throw_heads, coach, coin_choice_heads }
    }

    pub fn is_coin_throw_heads(&self) -> bool { self.coin_throw_heads }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn is_coin_choice_heads(&self) -> bool { self.coin_choice_heads }
}

impl IReport for ReportCoinThrow {
    fn get_id(&self) -> ReportId { ReportId::COIN_THROW }
}

impl ReportCoinThrow {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "coach": self.coach,
            "coinThrowHeads": self.coin_throw_heads,
            "coinChoiceHeads": self.coin_choice_heads,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            coach: json["coach"].as_str().unwrap_or("").to_string(),
            coin_throw_heads: json["coinThrowHeads"].as_bool().unwrap_or(false),
            coin_choice_heads: json["coinChoiceHeads"].as_bool().unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportCoinThrow {
        ReportCoinThrow::new(true, "CoachA".into(), false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::COIN_THROW);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "coinThrow");
    }

    #[test]
    fn get_coach() {
        assert_eq!(make().get_coach(), "CoachA");
    }

    #[test]
    fn coin_throw_heads() {
        assert!(make().is_coin_throw_heads());
    }

    #[test]
    fn coin_choice_tails() {
        assert!(!make().is_coin_choice_heads());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportCoinThrow::from_json(&json);
        assert_eq!(restored.coach, original.coach);
        assert_eq!(restored.coin_throw_heads, original.coin_throw_heads);
        assert_eq!(restored.coin_choice_heads, original.coin_choice_heads);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("coinThrow"));
    }
}
