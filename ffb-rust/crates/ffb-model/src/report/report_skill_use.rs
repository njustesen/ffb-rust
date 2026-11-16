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
    /// Java passes `skillUse == null` at a handful of sites (a declined Dump-Off, Wrestle, a
    /// Juggernaut that is not used): the JSON then carries `"skillUse": null`. `skill_use` keeps
    /// a placeholder for the accessors; this flag is what the serialiser honours.
    pub skill_use_is_null: bool,
    /// Java passes a null `skill` where the step looked the skill up as UNUSED and it no longer
    /// is (Catch of the Day's re-roll pass): the JSON carries `"skill": null`.
    pub skill_is_null: bool,
}

impl ReportSkillUse {
    pub fn new(
        player_id: Option<String>,
        skill: SkillId,
        used: bool,
        skill_use: SkillUse,
    ) -> Self {
        Self { player_id, skill, used, skill_use, skill_use_is_null: false, skill_is_null: false }
    }

    /// Java `new ReportSkillUse(playerId, null, used, skillUse)` — `skill` names the skill the
    /// step is about only for the accessors; the JSON writes null.
    pub fn with_null_skill(mut self) -> Self {
        self.skill_is_null = true;
        self
    }

    /// Java `new ReportSkillUse(playerId, skill, used, null)`.
    pub fn new_without_skill_use(player_id: Option<String>, skill: SkillId, used: bool) -> Self {
        Self { player_id, skill, used, skill_use: SkillUse::WOULD_NOT_HELP, skill_use_is_null: true, skill_is_null: false }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill(&self) -> SkillId { self.skill }
    pub fn is_used(&self) -> bool { self.used }
    pub fn get_skill_use(&self) -> SkillUse { self.skill_use }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "reportId": self.get_id().get_name(),
            "playerId": self.player_id,
            // Java `IJsonOption.SKILL.addTo(jsonObject, skill)` writes `skill.getName()` — the
            // display name ("Eye Gouge"), not the class name ("EyeGouge").
            "skill": if self.skill_is_null { serde_json::Value::Null }
                     else { serde_json::Value::from(self.skill.display_name()) },
            "used": self.used,
            "skillUse": if self.skill_use_is_null { serde_json::Value::Null }
                        else { serde_json::Value::from(self.skill_use.get_name()) },
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Self {
        Self {
            player_id: json["playerId"].as_str().map(str::to_string),
            skill: json["skill"].as_str().and_then(SkillId::from_class_name).unwrap_or(SkillId::Block),
            used: json["used"].as_bool().unwrap_or(false),
            skill_use: json["skillUse"].as_str().and_then(SkillUse::for_name).unwrap_or(SkillUse::WOULD_NOT_HELP),
            skill_use_is_null: json["skillUse"].is_null(),
            skill_is_null: json["skill"].is_null(),
        }
    }
}

impl IReport for ReportSkillUse {
    fn to_json(&self) -> Option<serde_json::Value> { Some(self.to_json_value()) }
    fn get_id(&self) -> ReportId { ReportId::SKILL_USE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportSkillUse {
        ReportSkillUse::new(Some("p1".into()), SkillId::Block, true, SkillUse::BRING_DOWN_OPPONENT)
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
