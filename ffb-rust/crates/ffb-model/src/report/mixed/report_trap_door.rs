use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportTrapDoor.java`.
#[derive(Debug, Clone)]
pub struct ReportTrapDoor {
    pub player_id: Option<String>,
    pub escaped: bool,
    pub roll: i32,
}

impl ReportTrapDoor {
    pub fn new(player_id: Option<String>, roll: i32, escaped: bool) -> Self {
        Self { player_id, escaped, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_escaped(&self) -> bool { self.escaped }
    pub fn get_roll(&self) -> i32 { self.roll }
}

impl IReport for ReportTrapDoor {
    fn get_id(&self) -> ReportId { ReportId::TRAP_DOOR }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportTrapDoor {
        ReportTrapDoor::new(Some("p1".into()), 4, true)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::TRAP_DOOR); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "trapDoor"); }

    #[test]
    fn get_roll() { assert_eq!(make().get_roll(), 4); }

    #[test]
    fn is_escaped() { assert!(make().is_escaped()); }
}
