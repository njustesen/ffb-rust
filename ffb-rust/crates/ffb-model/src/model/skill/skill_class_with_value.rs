/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillClassWithValue.
///
/// Java uses `Class<? extends Skill>` as the key; Rust uses the skill's string name
/// (the closest equivalent without runtime reflection).
pub struct SkillClassWithValue {
    /// The skill class name (replaces `Class<? extends Skill>` from Java).
    skill_class_name: String,
    value: Option<String>,
}

impl SkillClassWithValue {
    /// `SkillClassWithValue(Class<? extends Skill> skill)`.
    pub fn new(skill_class_name: impl Into<String>) -> Self {
        SkillClassWithValue { skill_class_name: skill_class_name.into(), value: None }
    }

    /// `SkillClassWithValue(Class<? extends Skill> skill, String value)`.
    pub fn with_value(skill_class_name: impl Into<String>, value: impl Into<String>) -> Self {
        SkillClassWithValue {
            skill_class_name: skill_class_name.into(),
            value: Some(value.into()),
        }
    }

    /// Java `getSkill()` — returns the class name string (Rust equivalent of the Class reference).
    pub fn get_skill(&self) -> &str {
        &self.skill_class_name
    }

    /// Java `getValue()`.
    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}
