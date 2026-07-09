use crate::enums::SkillId;
use crate::model::skill_use::SkillUse;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportSkillUse.java`.
/// `Skill` (Java) is represented as `SkillId`.
#[derive(Debug, Clone)]
pub struct ReportSkillUse {
    pub player_id: Option<String>,
    pub skill: SkillId,
    pub used: bool,
    pub skill_use: SkillUse,
}

impl ReportSkillUse {
    pub fn new(
        player_id: Option<String>,
        skill: SkillId,
        used: bool,
        skill_use: SkillUse,
    ) -> Self {
        Self { player_id, skill, used, skill_use }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill(&self) -> SkillId { self.skill }
    pub fn is_used(&self) -> bool { self.used }
    pub fn get_skill_use(&self) -> SkillUse { self.skill_use }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            "skill": self.skill.class_name(),
            "used": self.used,
            "skillUse": self.skill_use.get_name(),
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            skill: json["skill"].as_str().and_then(SkillId::from_class_name).unwrap_or(SkillId::Block),
            used: json["used"].as_bool().unwrap_or(false),
            skill_use: json["skillUse"].as_str().and_then(SkillUse::for_name).unwrap_or(SkillUse::WOULD_NOT_HELP),
        }
    }
}

impl IReport for ReportSkillUse {
    fn get_id(&self) -> ReportId { ReportId::SKILL_USE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillUse {
        ReportSkillUse::new(Some("p1".into()), SkillId::Block, true, SkillUse::BRING_DOWN_OPPONENT)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::SKILL_USE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "skillUse");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), Some("p1"));
        assert_eq!(r.get_skill(), SkillId::Block);
        assert!(r.is_used());
        assert_eq!(r.get_skill_use(), SkillUse::BRING_DOWN_OPPONENT);
    }

    #[test]
    fn not_used() {
        let r = ReportSkillUse::new(Some("p2".into()), SkillId::Block, false, SkillUse::WOULD_NOT_HELP);
        assert!(!r.is_used());
        assert_eq!(r.get_skill_use(), SkillUse::WOULD_NOT_HELP);
    }

    #[test]
    fn no_player_id() {
        let r = ReportSkillUse::new(None, SkillId::Block, true, SkillUse::STOP_OPPONENT);
        assert_eq!(r.get_player_id(), None);
        assert_eq!(r.get_skill_use(), SkillUse::STOP_OPPONENT);
    }

    #[test]
    fn serialization_round_trip() {
        let original = make();
        let json = original.to_json_value();
        let restored = ReportSkillUse::from_json(&json);
        assert_eq!(restored.player_id, original.player_id);
        assert_eq!(restored.skill, original.skill);
        assert_eq!(restored.used, original.used);
        assert_eq!(restored.skill_use, original.skill_use);
    }

    #[test]
    fn to_json_value_has_report_id() {
        let json = make().to_json_value();
        assert_eq!(json["reportId"].as_str(), Some("skillUse"));
    }
}
