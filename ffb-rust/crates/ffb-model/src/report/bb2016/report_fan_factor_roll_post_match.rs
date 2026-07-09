use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportFanFactorRollPostMatch.java`.
#[derive(Debug, Clone)]
pub struct ReportFanFactorRollPostMatch {
    pub fan_factor_roll_home: Vec<i32>,
    pub fan_factor_modifier_home: i32,
    pub fan_factor_roll_away: Vec<i32>,
    pub fan_factor_modifier_away: i32,
}

impl ReportFanFactorRollPostMatch {
    pub fn new(
        fan_factor_roll_home: Vec<i32>,
        fan_factor_modifier_home: i32,
        fan_factor_roll_away: Vec<i32>,
        fan_factor_modifier_away: i32,
    ) -> Self {
        Self { fan_factor_roll_home, fan_factor_modifier_home, fan_factor_roll_away, fan_factor_modifier_away }
    }

    pub fn get_fan_factor_roll_home(&self) -> &[i32] { &self.fan_factor_roll_home }
    pub fn get_fan_factor_modifier_home(&self) -> i32 { self.fan_factor_modifier_home }
    pub fn get_fan_factor_roll_away(&self) -> &[i32] { &self.fan_factor_roll_away }
    pub fn get_fan_factor_modifier_away(&self) -> i32 { self.fan_factor_modifier_away }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "fanFactorRollHome": self.fan_factor_roll_home,
            "fanFactorModifierHome": self.fan_factor_modifier_home,
            "fanFactorRollAway": self.fan_factor_roll_away,
            "fanFactorModifierAway": self.fan_factor_modifier_away,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            fan_factor_roll_home: json["fanFactorRollHome"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            fan_factor_modifier_home: json["fanFactorModifierHome"].as_i64().unwrap_or(0) as i32,
            fan_factor_roll_away: json["fanFactorRollAway"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            fan_factor_modifier_away: json["fanFactorModifierAway"].as_i64().unwrap_or(0) as i32,
        }
    }
}

impl IReport for ReportFanFactorRollPostMatch {
    fn get_id(&self) -> ReportId { ReportId::FAN_FACTOR_ROLL_POST_MATCH }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportFanFactorRollPostMatch {
        ReportFanFactorRollPostMatch::new(vec![3, 4], 1, vec![2, 5], -1)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::FAN_FACTOR_ROLL_POST_MATCH);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "fanFactorRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_fan_factor_roll_home(), &[3, 4]);
        assert_eq!(r.get_fan_factor_modifier_home(), 1);
        assert_eq!(r.get_fan_factor_modifier_away(), -1);
    }

    #[test]
    fn away_roll_stored() {
        let r = make();
        assert_eq!(r.get_fan_factor_roll_away(), &[2, 5]);
    }

    #[test]
    fn zero_modifiers() {
        let r = ReportFanFactorRollPostMatch::new(vec![1], 0, vec![2], 0);
        assert_eq!(r.get_fan_factor_modifier_home(), 0);
        assert_eq!(r.get_fan_factor_modifier_away(), 0);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportFanFactorRollPostMatch::from_json(&json);
        assert_eq!(restored.fan_factor_roll_home, original.fan_factor_roll_home);
        assert_eq!(restored.fan_factor_modifier_home, original.fan_factor_modifier_home);
        assert_eq!(restored.fan_factor_modifier_away, original.fan_factor_modifier_away);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("fanFactorRoll"));
    }
}
