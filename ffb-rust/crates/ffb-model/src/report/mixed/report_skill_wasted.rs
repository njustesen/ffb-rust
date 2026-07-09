use crate::enums::SkillId;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSkillWasted.java`.
#[derive(Debug, Clone)]
pub struct ReportSkillWasted {
    /// `fPlayerId`
    pub player_id: Option<String>,
    /// `fSkill`
    pub skill: Option<SkillId>,
}

impl ReportSkillWasted {
    pub fn new(player_id: Option<String>, skill: Option<SkillId>) -> Self {
        Self { player_id, skill }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill(&self) -> Option<SkillId> { self.skill }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "skill": self.skill.map(|s| s.class_name()),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            skill: json["skill"].as_str().and_then(SkillId::from_class_name),
        }
    }
}

impl IReport for ReportSkillWasted {
    fn get_id(&self) -> ReportId { ReportId::SKILL_WASTED }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillWasted {
        ReportSkillWasted::new(Some("p1".into()), None)
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::SKILL_WASTED); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "skillWasted"); }

    #[test]
    fn get_player_id() { assert_eq!(make().get_player_id(), Some("p1")); }

    #[test]
    fn get_skill_none() { assert!(make().get_skill().is_none()); }

    #[test]
    fn get_skill_some() {
        let r = ReportSkillWasted::new(Some("p2".into()), Some(SkillId::Dodge));
        assert_eq!(r.get_skill(), Some(SkillId::Dodge));
        assert_eq!(r.get_player_id(), Some("p2"));
    }

    #[test]
    fn serialization_round_trip() {
        let original = ReportSkillWasted::new(Some("p2".into()), Some(SkillId::Dodge));
        let json = original.to_json_value();
        let restored = ReportSkillWasted::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.skill, original.skill);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("skillWasted"));
    }
}
