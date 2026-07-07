use crate::enums::PlayerState;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportApothecaryRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportApothecaryRoll {
    pub player_id: Option<String>,
    pub casualty_roll: Vec<i32>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<String>,
    pub original_injury: Option<String>,
    pub casualty_modifiers: Vec<String>,
}

impl ReportApothecaryRoll {
    pub fn new(
        player_id: Option<String>,
        casualty_roll: Vec<i32>,
        player_state: Option<PlayerState>,
        serious_injury: Option<String>,
        original_injury: Option<String>,
        casualty_modifiers: Vec<String>,
    ) -> Self {
        Self { player_id, casualty_roll, player_state, serious_injury, original_injury, casualty_modifiers }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
    pub fn get_original_injury(&self) -> Option<&str> { self.original_injury.as_deref() }
    pub fn get_casualty_modifiers(&self) -> &[String] { &self.casualty_modifiers }
}

impl IReport for ReportApothecaryRoll {
    fn get_id(&self) -> ReportId { ReportId::APOTHECARY_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportApothecaryRoll {
        ReportApothecaryRoll::new(
            Some("p1".into()),
            vec![3, 4],
            None,
            Some("BROKEN_RIBS".into()),
            None,
            vec![],
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::APOTHECARY_ROLL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "apothecaryRoll"); }

    #[test]
    fn get_serious_injury() { assert_eq!(make().get_serious_injury(), Some("BROKEN_RIBS")); }

    #[test]
    fn get_player_id_and_casualty_roll() {
        assert_eq!(make().get_player_id(), Some("p1"));
        assert_eq!(make().get_casualty_roll(), &[3, 4]);
    }

    #[test]
    fn get_casualty_modifiers_and_original_injury() {
        assert_eq!(make().get_casualty_modifiers(), &[] as &[String]);
        assert_eq!(make().get_original_injury(), None);
    }
}
