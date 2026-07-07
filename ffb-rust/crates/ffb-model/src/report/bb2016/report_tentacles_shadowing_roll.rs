use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTentaclesShadowingRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportTentaclesShadowingRoll {
    pub skill: String,
    pub defender_id: String,
    pub roll: Vec<i32>,
    pub successful: bool,
    pub minimum_roll: i32,
    pub re_rolled: bool,
}

impl ReportTentaclesShadowingRoll {
    pub fn new(
        skill: String,
        defender_id: String,
        roll: Vec<i32>,
        successful: bool,
        minimum_roll: i32,
        re_rolled: bool,
    ) -> Self {
        Self { skill, defender_id, roll, successful, minimum_roll, re_rolled }
    }

    pub fn get_skill(&self) -> &str { &self.skill }
    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn get_roll(&self) -> &[i32] { &self.roll }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_re_rolled(&self) -> bool { self.re_rolled }
}

impl IReport for ReportTentaclesShadowingRoll {
    fn get_id(&self) -> ReportId { ReportId::TENTACLES_SHADOWING_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTentaclesShadowingRoll {
        ReportTentaclesShadowingRoll::new("Tentacles".into(), "d1".into(), vec![3, 4], false, 5, false)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::TENTACLES_SHADOWING_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "tentaclesShadowingRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_skill(), "Tentacles");
        assert_eq!(r.get_defender_id(), "d1");
        assert!(!r.is_successful());
    }

    #[test]
    fn minimum_roll_and_roll_values() {
        let r = make();
        assert_eq!(r.get_minimum_roll(), 5);
        assert_eq!(r.get_roll(), &[3, 4]);
    }

    #[test]
    fn rerolled_and_successful() {
        let r = ReportTentaclesShadowingRoll::new("Shadowing".into(), "d2".into(), vec![6], true, 4, true);
        assert!(r.is_successful());
        assert!(r.is_re_rolled());
        assert_eq!(r.get_skill(), "Shadowing");
    }
}
