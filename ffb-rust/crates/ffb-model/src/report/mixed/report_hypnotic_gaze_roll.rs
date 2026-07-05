use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::report::report_skill_roll::ReportSkillRoll;

/// 1:1 translation of `ReportHypnoticGazeRoll.java` — extends ReportSkillRoll.
#[derive(Debug, Clone)]
pub struct ReportHypnoticGazeRoll {
    pub base: ReportSkillRoll,
    pub defender_id: Option<String>,
}

impl ReportHypnoticGazeRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        roll: i32,
        minimum_roll: i32,
        re_rolled: bool,
        defender_id: Option<String>,
    ) -> Self {
        Self {
            base: ReportSkillRoll::new(player_id, successful, roll, minimum_roll, re_rolled, vec![]),
            defender_id,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn is_successful(&self) -> bool { self.base.is_successful() }
    pub fn get_roll(&self) -> i32 { self.base.get_roll() }
    pub fn get_minimum_roll(&self) -> i32 { self.base.get_minimum_roll() }
    pub fn is_re_rolled(&self) -> bool { self.base.is_re_rolled() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportHypnoticGazeRoll {
    fn get_id(&self) -> ReportId { ReportId::HYPNOTIC_GAZE_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportHypnoticGazeRoll {
        ReportHypnoticGazeRoll::new(Some("p1".into()), true, 4, 2, false, Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::HYPNOTIC_GAZE_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "hypnoticGazeRoll"); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }
}
