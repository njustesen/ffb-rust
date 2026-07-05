use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDedicatedFans.java`.
#[derive(Debug, Clone)]
pub struct ReportDedicatedFans {
    pub roll_home: i32,
    pub modifier_home: i32,
    pub roll_away: i32,
    pub modifier_away: i32,
    pub conceded_team: Option<String>,
    pub conceded: bool,
}

impl ReportDedicatedFans {
    pub fn new(
        roll_home: i32,
        modifier_home: i32,
        roll_away: i32,
        modifier_away: i32,
        conceded_team: Option<String>,
        conceded: bool,
    ) -> Self {
        Self { roll_home, modifier_home, roll_away, modifier_away, conceded_team, conceded }
    }

    pub fn get_roll_home(&self) -> i32 { self.roll_home }
    pub fn get_modifier_home(&self) -> i32 { self.modifier_home }
    pub fn get_roll_away(&self) -> i32 { self.roll_away }
    pub fn get_modifier_away(&self) -> i32 { self.modifier_away }
    pub fn get_conceded_team(&self) -> Option<&str> { self.conceded_team.as_deref() }
    pub fn is_conceded(&self) -> bool { self.conceded }
}

impl IReport for ReportDedicatedFans {
    fn get_id(&self) -> ReportId { ReportId::DEDICATED_FANS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDedicatedFans {
        ReportDedicatedFans::new(3, 1, 2, 0, Some("away".into()), true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::DEDICATED_FANS); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "dedicatedFans"); }

    #[test]
    fn get_roll_home() { assert_eq!(make().get_roll_home(), 3); }
}
