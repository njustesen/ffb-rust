use serde::{Deserialize, Serialize};
use crate::enums::{SkillCategory, SkillUsageType, DeclareCondition};

pub use crate::enums::SkillId;

/// A skill value annotation (e.g. Mighty Blow "+1", Animosity condition).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillWithValue {
    pub skill_id: SkillId,
    pub value: Option<String>,
}

impl SkillWithValue {
    pub fn new(skill_id: SkillId) -> Self {
        SkillWithValue { skill_id, value: None }
    }

    pub fn with_value(skill_id: SkillId, value: impl Into<String>) -> Self {
        SkillWithValue { skill_id, value: Some(value.into()) }
    }
}

/// Static definition of a skill (stored in the skill table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: SkillId,
    pub name: String,
    pub category: SkillCategory,
    pub usage_type: SkillUsageType,
    pub declare_condition: Option<DeclareCondition>,
}

impl SkillDef {
    pub fn new(
        id: SkillId,
        name: impl Into<String>,
        category: SkillCategory,
        usage_type: SkillUsageType,
    ) -> Self {
        SkillDef {
            id,
            name: name.into(),
            category,
            usage_type,
            declare_condition: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{SkillCategory, SkillUsageType};

    #[test]
    fn skill_with_value_serde() {
        let sw = SkillWithValue::with_value(SkillId::Block, "2");
        let json = serde_json::to_string(&sw).unwrap();
        let back: SkillWithValue = serde_json::from_str(&json).unwrap();
        assert_eq!(sw, back);
    }
}
