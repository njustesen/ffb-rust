use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportPenaltyShootout.java`.
#[derive(Debug, Clone)]
pub struct ReportPenaltyShootout {
    pub roll_home: i32,
    pub re_rolls_left_home: i32,
    pub roll_away: i32,
    pub re_rolls_left_away: i32,
}

impl ReportPenaltyShootout {
    pub fn new(roll_home: i32, re_rolls_left_home: i32, roll_away: i32, re_rolls_left_away: i32) -> Self {
        Self { roll_home, re_rolls_left_home, roll_away, re_rolls_left_away }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_re_rolls_left_home(&self) -> i32 { self.re_rolls_left_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_re_rolls_left_away(&self) -> i32 { self.re_rolls_left_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "rollHome": self.roll_home,
            "reRollsLeftHome": self.re_rolls_left_home,
            "rollAway": self.roll_away,
            "reRollsLeftAway": self.re_rolls_left_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            roll_home: json["rollHome"].as_i64().unwrap_or(0) as i32,
            re_rolls_left_home: json["reRollsLeftHome"].as_i64().unwrap_or(0) as i32,
            roll_away: json["rollAway"].as_i64().unwrap_or(0) as i32,
            re_rolls_left_away: json["reRollsLeftAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportPenaltyShootout {
    fn get_id(&self) -> ReportId { ReportId::PENALTY_SHOOTOUT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportPenaltyShootout {
        ReportPenaltyShootout::new(4, 2, 3, 1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::PENALTY_SHOOTOUT);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "penaltyShootout");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_roll_home(), 4);
        assert_eq!(r.get_re_rolls_left_home(), 2);
        assert_eq!(r.get_roll_away(), 3);
        assert_eq!(r.get_re_rolls_left_away(), 1);
    }

    #[test]
    fn zero_rerolls_left() {
        let r = ReportPenaltyShootout::new(5, 0, 6, 0);
        assert_eq!(r.get_re_rolls_left_home(), 0);
        assert_eq!(r.get_re_rolls_left_away(), 0);
    }

    #[test]
    fn asymmetric_values() {
        let r = ReportPenaltyShootout::new(1, 3, 6, 0);
        assert_eq!(r.get_roll_home(), 1);
        assert_eq!(r.get_roll_away(), 6);
        assert_eq!(r.get_re_rolls_left_home(), 3);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportPenaltyShootout::from_json(&json);
        assert_eq!(restored.roll_home, original.roll_home);
        assert_eq!(restored.re_rolls_left_home, original.re_rolls_left_home);
        assert_eq!(restored.roll_away, original.roll_away);
        assert_eq!(restored.re_rolls_left_away, original.re_rolls_left_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("penaltyShootout"));
    }
}
