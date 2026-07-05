use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSpectators.java`.
#[derive(Debug, Clone)]
pub struct ReportSpectators {
    pub spectator_roll_home: Vec<i32>,
    pub spectators_home: i32,
    pub fame_home: i32,
    pub spectator_roll_away: Vec<i32>,
    pub spectators_away: i32,
    pub fame_away: i32,
}

impl ReportSpectators {
    pub fn new(
        spectator_roll_home: Vec<i32>,
        spectators_home: i32,
        fame_home: i32,
        spectator_roll_away: Vec<i32>,
        spectators_away: i32,
        fame_away: i32,
    ) -> Self {
        Self { spectator_roll_home, spectators_home, fame_home, spectator_roll_away, spectators_away, fame_away }
    }

    pub fn get_spectator_roll_home(&self) -> &[i32] { &self.spectator_roll_home }
    pub fn get_spectators_home(&self) -> i32 { self.spectators_home }
    pub fn get_fame_home(&self) -> i32 { self.fame_home }
    pub fn get_spectator_roll_away(&self) -> &[i32] { &self.spectator_roll_away }
    pub fn get_spectators_away(&self) -> i32 { self.spectators_away }
    pub fn get_fame_away(&self) -> i32 { self.fame_away }
}

impl IReport for ReportSpectators {
    fn get_id(&self) -> ReportId { ReportId::SPECTATORS }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSpectators {
        ReportSpectators::new(vec![3, 4], 35000, 1, vec![2, 5], 20000, 0)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SPECTATORS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "spectators");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_spectators_home(), 35000);
        assert_eq!(r.get_fame_home(), 1);
        assert_eq!(r.get_spectators_away(), 20000);
    }
}
