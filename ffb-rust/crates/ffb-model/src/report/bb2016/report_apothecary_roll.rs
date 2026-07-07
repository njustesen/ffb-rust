use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::PlayerState;

/// 1:1 translation of `ReportApothecaryRoll.java`.
#[derive(Debug, Clone)]
pub struct ReportApothecaryRoll {
    pub player_id: String,
    pub casualty_roll: Vec<i32>,
    pub player_state: Option<PlayerState>,
    pub serious_injury: Option<String>,
}

impl ReportApothecaryRoll {
    pub fn new(
        player_id: String,
        casualty_roll: Vec<i32>,
        player_state: Option<PlayerState>,
        serious_injury: Option<String>,
    ) -> Self {
        Self { player_id, casualty_roll, player_state, serious_injury }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_player_state(&self) -> Option<PlayerState> { self.player_state }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
}

impl IReport for ReportApothecaryRoll {
    fn get_id(&self) -> ReportId { ReportId::APOTHECARY_ROLL }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportApothecaryRoll {
        ReportApothecaryRoll::new("p1".into(), vec![3, 4], None, None)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::APOTHECARY_ROLL);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "apothecaryRoll");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_casualty_roll(), &[3, 4]);
        assert_eq!(r.get_serious_injury(), None);
    }

    #[test]
    fn serious_injury_stored() {
        let r = ReportApothecaryRoll::new("p2".into(), vec![5, 6], None, Some("NIGGLING_INJURY".into()));
        assert_eq!(r.get_serious_injury(), Some("NIGGLING_INJURY"));
    }

    #[test]
    fn player_state_none_by_default() {
        let r = make();
        assert!(r.get_player_state().is_none());
    }
}
