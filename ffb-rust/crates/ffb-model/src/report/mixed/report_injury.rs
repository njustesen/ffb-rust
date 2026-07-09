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

impl ReportInjury {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "defenderId": self.defender_id,
            "injuryType": self.injury_type,
            "armorBroken": self.armor_broken,
            "armorRoll": self.armor_roll,
            "injuryRoll": self.injury_roll,
            "casualtyRoll": self.casualty_roll,
            "seriousInjury": self.serious_injury,
            "casualtyRollDecay": self.casualty_roll_decay,
            "seriousInjuryDecay": self.serious_injury_decay,
            "seriousInjuryOld": self.original_injury,
            "injury": self.injury.map(|s| s.id()),
            "injuryDecay": self.injury_decay.map(|s| s.id()),
            "attackerId": self.attacker_id,
            "armorModifiers": self.armor_modifiers,
            "injuryModifiers": self.injury_modifiers,
            "casualtyModifiers": self.casualty_modifiers,
            "skipInjuryParts": self.skip,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            attacker_id: json["attackerId"].as_str().map(str::to_string),
            defender_id: json["defenderId"].as_str().map(str::to_string),
            injury_type: json["injuryType"].as_str().unwrap_or("").to_string(),
            armor_broken: json["armorBroken"].as_bool().unwrap_or(false),
            armor_modifiers: json["armorModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            armor_roll: json["armorRoll"].as_array().map(|a| a.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect()).unwrap_or_default(),
            injury_modifiers: json["injuryModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            injury_roll: json["injuryRoll"].as_array().map(|a| a.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect()).unwrap_or_default(),
            casualty_roll: json["casualtyRoll"].as_array().map(|a| a.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect()).unwrap_or_default(),
            serious_injury: json["seriousInjury"].as_str().map(str::to_string),
            casualty_roll_decay: json["casualtyRollDecay"].as_array().map(|a| a.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect()).unwrap_or_default(),
            serious_injury_decay: json["seriousInjuryDecay"].as_str().map(str::to_string),
            original_injury: json["seriousInjuryOld"].as_str().map(str::to_string),
            injury: json["injury"].as_u64().map(|n| PlayerState::new(n as u32)),
            injury_decay: json["injuryDecay"].as_u64().map(|n| PlayerState::new(n as u32)),
            casualty_modifiers: json["casualtyModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            skip: json["skipInjuryParts"].as_str().unwrap_or("").to_string(),
        }
    }
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

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportInjury::from_json(&json);
        assert_eq!(restored.attacker_id, original.attacker_id);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.injury_type, original.injury_type);
        assert_eq!(restored.armor_broken, original.armor_broken);
        assert_eq!(restored.armor_roll, original.armor_roll);
        assert_eq!(restored.injury_roll, original.injury_roll);
        assert_eq!(restored.skip, original.skip);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("injury"));
    }
}
