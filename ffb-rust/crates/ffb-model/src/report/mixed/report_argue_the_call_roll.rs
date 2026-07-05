use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportArgueTheCallRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportArgueTheCallRoll {
    pub player_id: Option<String>,
    pub successful: bool,
    pub coach_banned: bool,
    pub roll: i32,
    pub stays_on_pitch: bool,
    pub friends_with_ref: bool,
    pub biased_refs: i32,
}

impl ReportArgueTheCallRoll {
    pub fn new(
        player_id: Option<String>,
        successful: bool,
        coach_banned: bool,
        roll: i32,
        stays_on_pitch: bool,
        friends_with_ref: bool,
        biased_refs: i32,
    ) -> Self {
        Self { player_id, successful, coach_banned, roll, stays_on_pitch, friends_with_ref, biased_refs }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_successful(&self) -> bool { self.successful }
    pub fn is_coach_banned(&self) -> bool { self.coach_banned }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn is_stays_on_pitch(&self) -> bool { self.stays_on_pitch }
    pub fn is_friends_with_ref(&self) -> bool { self.friends_with_ref }
    pub fn get_biased_refs(&self) -> i32 { self.biased_refs }
}

impl IReport for ReportArgueTheCallRoll {
    fn get_id(&self) -> ReportId { ReportId::ARGUE_THE_CALL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportArgueTheCallRoll {
        ReportArgueTheCallRoll::new(Some("p1".into()), true, false, 5, true, false, 1)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::ARGUE_THE_CALL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "argueTheCall"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 5); }
}
