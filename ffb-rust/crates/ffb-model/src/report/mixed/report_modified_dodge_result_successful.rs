use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportModifiedDodgeResultSuccessful.java`.
#[derive(Debug, Clone)]
pub struct ReportModifiedDodgeResultSuccessful {
    pub skill_id: Option<SkillId>,
}

impl ReportModifiedDodgeResultSuccessful {
    pub fn new(skill_id: Option<SkillId>) -> Self {
        Self { skill_id }
    }

    pub fn get_skill_id(&self) -> Option<SkillId> { self.skill_id }
}

impl IReport for ReportModifiedDodgeResultSuccessful {
    fn get_id(&self) -> ReportId { ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL }
}

impl ReportModifiedDodgeResultSuccessful {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "skill": self.skill_id.map(|s| s.class_name()),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            skill_id: json["skill"].as_str().and_then(SkillId::from_class_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportModifiedDodgeResultSuccessful {
        ReportModifiedDodgeResultSuccessful::new(None)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::MODIFIED_DODGE_RESULT_SUCCESSFUL); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "modifiedDodgeResultSuccessful"); }

    #[test]
    fn get_skill_id_none() { assert!(make().get_skill_id().is_none()); }

    #[test]
    fn get_skill_id_some() {
        let r = ReportModifiedDodgeResultSuccessful::new(Some(SkillId::Dodge));
        assert_eq!(r.get_skill_id(), Some(SkillId::Dodge));
    }

    #[test]
    fn none_construction() {
        let r = ReportModifiedDodgeResultSuccessful::new(None);
        assert!(r.get_skill_id().is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportModifiedDodgeResultSuccessful::new(Some(SkillId::Dodge));
        let json = original.to_json_value();
        let restored = ReportModifiedDodgeResultSuccessful::from_json(&json);
        assert_eq!(restored.skill_id, original.skill_id);
    }

    #[test]
    fn serialization_round_trip_none() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportModifiedDodgeResultSuccessful::from_json(&json);
        assert_eq!(restored.skill_id, original.skill_id);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("modifiedDodgeResultSuccessful"));
    }
}
