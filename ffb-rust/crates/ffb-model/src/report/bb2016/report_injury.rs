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

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "defenderId": self.defender_id,
            "injuryType": self.injury_type,
            "armorBroken": self.armor_broken,
            "armorModifiers": self.armor_modifier_names,
            "armorRoll": self.armor_roll,
            "injuryModifiers": self.injury_modifier_names,
            "injuryRoll": self.injury_roll,
            "casualtyRoll": self.casualty_roll,
            "seriousInjury": self.serious_injury,
            "casualtyRollDecay": self.casualty_roll_decay,
            "seriousInjuryDecay": self.serious_injury_decay,
            "injury": self.injury.map(|ps| ps.id()),
            "injuryDecay": self.injury_decay.map(|ps| ps.id()),
            "attackerId": self.attacker_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            attacker_id: json["attackerId"].as_str().map(str::to_string),
            defender_id: json["defenderId"].as_str().unwrap_or("").to_string(),
            injury_type: json["injuryType"].as_str().unwrap_or("").to_string(),
            armor_broken: json["armorBroken"].as_bool().unwrap_or(false),
            armor_modifier_names: json["armorModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            armor_roll: json["armorRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            injury_modifier_names: json["injuryModifiers"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            injury_roll: json["injuryRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            casualty_roll: json["casualtyRoll"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            serious_injury: json["seriousInjury"].as_str().map(str::to_string),
            casualty_roll_decay: json["casualtyRollDecay"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()).unwrap_or_default(),
            serious_injury_decay: json["seriousInjuryDecay"].as_str().map(str::to_string),
            injury: json["injury"].as_u64().map(|n| PlayerState::new(n as u32)),
            injury_decay: json["injuryDecay"].as_u64().map(|n| PlayerState::new(n as u32)),
        }
    }
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

    #[test]
    fn armor_and_injury_rolls_stored() {
        let r = make();
        assert_eq!(r.get_armor_roll(), &[3, 4]);
        assert_eq!(r.get_injury_roll(), &[2, 5]);
        assert_eq!(r.get_injury_type(), "casualty");
    }

    #[test]
    fn no_attacker_and_serious_injury() {
        let r = ReportInjury::new(
            "d2".into(), "stun".into(), false, vec![], vec![2, 3],
            vec![], vec![1], vec![4, 5], Some("BADLY_HURT".into()),
            vec![], None, None, None, None,
        );
        assert_eq!(r.get_attacker_id(), None);
        assert_eq!(r.get_serious_injury(), Some("BADLY_HURT"));
        assert!(!r.is_armor_broken());
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportInjury::from_json(&json);
        assert_eq!(restored.defender_id, original.defender_id);
        assert_eq!(restored.armor_broken, original.armor_broken);
        assert_eq!(restored.injury_type, original.injury_type);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("injury"));
    }
}
