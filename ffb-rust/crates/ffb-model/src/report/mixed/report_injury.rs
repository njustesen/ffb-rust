use crate::enums::PlayerState;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportInjury.java`.
#[derive(Debug, Clone)]
pub struct ReportInjury {
    pub attacker_id: Option<String>,
    pub defender_id: Option<String>,
    pub injury_type: String,
    pub armor_broken: bool,
    pub armor_modifiers: Vec<String>,
    pub armor_roll: Vec<i32>,
    pub injury_modifiers: Vec<String>,
    pub injury_roll: Vec<i32>,
    pub casualty_roll: Vec<i32>,
    pub serious_injury: Option<String>,
    pub casualty_roll_decay: Vec<i32>,
    pub serious_injury_decay: Option<String>,
    pub original_injury: Option<String>,
    pub injury: Option<PlayerState>,
    pub injury_decay: Option<PlayerState>,
    pub casualty_modifiers: Vec<String>,
    pub skip: String,
}

impl ReportInjury {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attacker_id: Option<String>,
        defender_id: Option<String>,
        injury_type: String,
        armor_broken: bool,
        armor_modifiers: Vec<String>,
        armor_roll: Vec<i32>,
        injury_modifiers: Vec<String>,
        injury_roll: Vec<i32>,
        casualty_roll: Vec<i32>,
        serious_injury: Option<String>,
        casualty_roll_decay: Vec<i32>,
        serious_injury_decay: Option<String>,
        original_injury: Option<String>,
        injury: Option<PlayerState>,
        injury_decay: Option<PlayerState>,
        casualty_modifiers: Vec<String>,
        skip: String,
    ) -> Self {
        Self {
            attacker_id, defender_id, injury_type, armor_broken, armor_modifiers, armor_roll,
            injury_modifiers, injury_roll, casualty_roll, serious_injury, casualty_roll_decay,
            serious_injury_decay, original_injury, injury, injury_decay, casualty_modifiers, skip,
        }
    }

    pub fn get_attacker_id(&self) -> Option<&str> { self.attacker_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
    pub fn get_injury_type(&self) -> &str { &self.injury_type }
    pub fn is_armor_broken(&self) -> bool { self.armor_broken }
    pub fn get_armor_modifiers(&self) -> &[String] { &self.armor_modifiers }
    pub fn get_armor_roll(&self) -> &[i32] { &self.armor_roll }
    pub fn get_injury_modifiers(&self) -> &[String] { &self.injury_modifiers }
    pub fn get_injury_roll(&self) -> &[i32] { &self.injury_roll }
    pub fn get_casualty_roll(&self) -> &[i32] { &self.casualty_roll }
    pub fn get_serious_injury(&self) -> Option<&str> { self.serious_injury.as_deref() }
    pub fn get_casualty_roll_decay(&self) -> &[i32] { &self.casualty_roll_decay }
    pub fn get_serious_injury_decay(&self) -> Option<&str> { self.serious_injury_decay.as_deref() }
    pub fn get_original_injury(&self) -> Option<&str> { self.original_injury.as_deref() }
    pub fn get_injury(&self) -> Option<PlayerState> { self.injury }
    pub fn get_injury_decay(&self) -> Option<PlayerState> { self.injury_decay }
    pub fn get_casualty_modifiers(&self) -> &[String] { &self.casualty_modifiers }
    pub fn get_skip(&self) -> &str { &self.skip }
}

impl IReport for ReportInjury {
    fn get_id(&self) -> ReportId { ReportId::INJURY }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportInjury {
        ReportInjury::new(
            Some("a1".into()), Some("d1".into()), "REGULAR".into(), true,
            vec![], vec![3, 4], vec![], vec![5], vec![], None,
            vec![], None, None, None, None, vec![], "none".into(),
        )
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::INJURY); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "injury"); }

    #[test]
    fn get_injury_type() { assert_eq!(make().get_injury_type(), "REGULAR"); }

    #[test]
    fn get_attacker_and_defender_id() {
        assert_eq!(make().get_attacker_id(), Some("a1"));
        assert_eq!(make().get_defender_id(), Some("d1"));
    }

    #[test]
    fn is_armor_broken_and_armor_roll() {
        assert!(make().is_armor_broken());
        assert_eq!(make().get_armor_roll(), &[3, 4]);
    }
}
