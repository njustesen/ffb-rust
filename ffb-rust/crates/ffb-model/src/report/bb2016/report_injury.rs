use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;
use crate::enums::{PlayerState, PS_BADLY_HURT};

/// 1:1 translation of `ReportInjury.java`.
#[derive(Debug, Clone)]
pub struct ReportInjury {
    pub attacker_id: Option<String>,
    pub defender_id: String,
    pub injury_type: String,
    pub armor_broken: bool,
    pub armor_modifier_names: Vec<String>,
    pub armor_roll: Vec<i32>,
    pub injury_modifier_names: Vec<String>,
    pub injury_roll: Vec<i32>,
    pub casualty_roll: Vec<i32>,
    pub serious_injury: Option<String>,
    pub casualty_roll_decay: Vec<i32>,
    pub serious_injury_decay: Option<String>,
    pub injury: Option<PlayerState>,
    pub injury_decay: Option<PlayerState>,
}

impl ReportInjury {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        defender_id: String,
        injury_type: String,
        armor_broken: bool,
        armor_modifier_names: Vec<String>,
        armor_roll: Vec<i32>,
        injury_modifier_names: Vec<String>,
        injury_roll: Vec<i32>,
        casualty_roll: Vec<i32>,
        serious_injury: Option<String>,
        casualty_roll_decay: Vec<i32>,
        serious_injury_decay: Option<String>,
        injury: Option<PlayerState>,
        injury_decay: Option<PlayerState>,
        attacker_id: Option<String>,
    ) -> Self {
        Self {
            attacker_id,
            defender_id,
            injury_type,
            armor_broken,
            armor_modifier_names,
            armor_roll,
            injury_modifier_names,
            injury_roll,
            casualty_roll,
            serious_injury,
            casualty_roll_decay,
            serious_injury_decay,
            injury,
            injury_decay,
        }
    }

    pub fn get_attacker_id(&self) -> Option<&str> { self.attacker_id.as_deref() }
    pub fn get_defender_id(&self) -> &str { &self.defender_id }
    pub fn get_injury_type(&self) -> &str { &self.injury_type }
    pub fn is_armor_broken(&self) -> bool { self.armor_broken }
    pub fn get_armor_modifier_names(&self) -> &[String] { &self.armor_modifier_names }
    pub fn get_armor_roll(&self) -> &[i32] { &self.armor_roll }
    pub fn get_injury_modifier_names(&self) -> &[String] { &self.injury_modifier_names }
    pub fn get_injury_roll(&self) -> &[i32] { &self.injury_roll }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
    pub fn get_casualty_roll_decay(&self) -> &[i32] { &self.casualty_roll_decay }
    pub fn get_serious_injury_decay(&self) -> Option<&str> { self.serious_injury_decay.as_deref() }
    pub fn get_injury(&self) -> Option<PlayerState> { self.injury }
    pub fn get_injury_decay(&self) -> Option<PlayerState> { self.injury_decay }
}

impl IReport for ReportInjury {
    fn get_id(&self) -> ReportId { ReportId::INJURY }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInjury {
        ReportInjury::new(
            "defender1".into(),
            "casualty".into(),
            true,
            vec![],
            vec![3, 4],
            vec![],
            vec![2, 5],
            vec![],
            None,
            vec![],
            None,
            Some(PlayerState::new(PS_BADLY_HURT)),
            None,
            Some("attacker1".into()),
        )
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::INJURY);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "injury");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_defender_id(), "defender1");
        assert!(r.is_armor_broken());
        assert_eq!(r.get_attacker_id(), Some("attacker1"));
    }
}
