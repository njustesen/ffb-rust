use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportAnimalSavagery.java`.
#[derive(Debug, Clone)]
pub struct ReportAnimalSavagery {
    pub attacker_id: Option<String>,
    pub defender_id: Option<String>,
}

impl ReportAnimalSavagery {
    pub fn new(attacker_id: Option<String>, defender_id: Option<String>) -> Self {
        Self { attacker_id, defender_id }
    }

    pub fn get_attacker_id(&self) -> Option<&str> { self.attacker_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportAnimalSavagery {
    fn get_id(&self) -> ReportId { ReportId::ANIMAL_SAVAGERY }
}

impl ReportAnimalSavagery {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "attackerId": self.attacker_id,
            "defenderId": self.defender_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            attacker_id: json["attackerId"].as_str().map(str::to_string),
            defender_id: json["defenderId"].as_str().map(str::to_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportAnimalSavagery {
        ReportAnimalSavagery::new(Some("a1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::ANIMAL_SAVAGERY); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "animalSavagery"); }

    #[test]
    fn get_attacker_id() { assert_eq!(make().get_attacker_id(), Some("a1")); }

    #[test]
    fn get_defender_id() { assert_eq!(make().get_defender_id(), Some("d1")); }

    #[test]
    fn none_ids() {
        let r = ReportAnimalSavagery::new(None, None);
        assert_eq!(r.get_attacker_id(), None);
        assert_eq!(r.get_defender_id(), None);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportAnimalSavagery::from_json(&json);
        assert_eq!(restored.attacker_id, original.attacker_id);
        assert_eq!(restored.defender_id, original.defender_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("animalSavagery"));
    }
}
