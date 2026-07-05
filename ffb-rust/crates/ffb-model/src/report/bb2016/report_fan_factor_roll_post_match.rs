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
}
