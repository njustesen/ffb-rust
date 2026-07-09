/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillWithValue.
use crate::model::skill::skill::Skill;

/// Associates a `Skill` with an optional string value
/// (e.g. Mighty Blow "+1", Animosity target condition).
pub struct SkillWithValue {
    skill: Skill,
    value: Option<String>,
}

impl SkillWithValue {
    /// `SkillWithValue(Skill skill)` — no value.
    pub fn new(skill: Skill) -> Self {
        SkillWithValue { skill, value: None }
    }

    /// `SkillWithValue(Skill skill, String value)`.
    pub fn with_value(skill: Skill, value: impl Into<String>) -> Self {
        SkillWithValue { skill, value: Some(value.into()) }
    }

    /// Java `getSkill()`.
    pub fn get_skill(&self) -> &Skill {
        &self.skill
    }

    /// Java `getValue()` — returns `Optional<String>` in Java; here `Option<&str>`.
    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillCategory;

    fn make_skill(name: &str) -> Skill {
        Skill::new(name, SkillCategory::General)
    }

    #[test]
    fn new_has_no_value() {
        let swv = SkillWithValue::new(make_skill("Block"));
        assert!(swv.get_value().is_none());
    }

    #[test]
    fn with_value_stores_value() {
        let swv = SkillWithValue::with_value(make_skill("Mighty Blow"), "+1");
        assert_eq!(swv.get_value(), Some("+1"));
    }

    #[test]
    fn get_skill_returns_correct_skill() {
        let swv = SkillWithValue::new(make_skill("Dodge"));
        assert_eq!(swv.get_skill().get_name(), "Dodge");
    }

    #[test]
    fn with_value_get_skill_name_matches() {
        let swv = SkillWithValue::with_value(make_skill("Tackle"), "special");
        assert_eq!(swv.get_skill().get_name(), "Tackle");
    }
}
