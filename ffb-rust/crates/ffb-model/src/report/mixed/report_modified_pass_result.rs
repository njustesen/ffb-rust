use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportModifiedPassResult.java`.
#[derive(Debug, Clone)]
pub struct ReportModifiedPassResult {
    pub skill_id: Option<SkillId>,
    pub pass_result: String,
}

impl ReportModifiedPassResult {
    pub fn new(skill_id: Option<SkillId>, pass_result: String) -> Self {
        Self { skill_id, pass_result }
    }

    pub fn get_skill_id(&self) -> Option<SkillId> { self.skill_id }
    pub fn get_pass_result(&self) -> &str { &self.pass_result }
}

impl IReport for ReportModifiedPassResult {
    fn get_id(&self) -> ReportId { ReportId::MODIFIED_PASS_RESULT }
}

impl ReportModifiedPassResult {
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "skill": self.skill_id.map(|s| s.class_name()),
            "passResult": self.pass_result,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            skill_id: json["skill"].as_str().and_then(SkillId::from_class_name),
            pass_result: json["passResult"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportModifiedPassResult {
        ReportModifiedPassResult::new(None, "ACCURATE".into())
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportModifiedPassResult::new(Some(SkillId::Pass), "ACCURATE".into());
        let json = original.to_json_value();
        let restored = ReportModifiedPassResult::from_json(&json);
        assert_eq!(restored.skill_id, original.skill_id);
        assert_eq!(restored.pass_result, original.pass_result);
    }

    #[test]
    fn serialization_round_trip_no_skill() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportModifiedPassResult::from_json(&json);
        assert_eq!(restored.skill_id, original.skill_id);
        assert_eq!(restored.pass_result, original.pass_result);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("modifiedPassResult"));
    }
}
