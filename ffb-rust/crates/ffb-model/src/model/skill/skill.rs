use serde::{Deserialize, Serialize};
use crate::model::skill_def::SkillDef;

/// 1:1 translation of com.fumbbl.ffb.model.skill.Skill.
/// Wraps a SkillDef for use in model contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub skill_def: SkillDef,
}

impl Skill {
    pub fn new(skill_def: SkillDef) -> Self { Self { skill_def } }
    pub fn get_name(&self) -> &str { &self.skill_def.name }
    pub fn get_id(&self) -> crate::enums::SkillId { self.skill_def.id }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::skill_def::SkillDef;
    use crate::enums::{SkillId, SkillCategory, SkillUsageType};

    fn make_skill_def() -> SkillDef {
        SkillDef {
            id: SkillId::Block,
            name: "Block".to_string(),
            category: SkillCategory::General,
            usage_type: SkillUsageType::Regular,
            declare_condition: None,
        }
    }

    #[test]
    fn get_name_returns_block() {
        let s = Skill::new(make_skill_def());
        assert_eq!(s.get_name(), "Block");
    }

    #[test]
    fn get_id_returns_block() {
        let s = Skill::new(make_skill_def());
        assert_eq!(s.get_id(), SkillId::Block);
    }
}
