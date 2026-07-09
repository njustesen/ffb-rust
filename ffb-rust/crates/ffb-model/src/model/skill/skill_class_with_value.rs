use serde::{Deserialize, Serialize};
use crate::enums::SkillId;

/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillClassWithValue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillClassWithValue {
    pub skill_id: SkillId,
    pub value: i32,
}

impl SkillClassWithValue {
    pub fn new(skill_id: SkillId, value: i32) -> Self { Self { skill_id, value } }
    pub fn get_skill_id(&self) -> SkillId { self.skill_id }
    pub fn get_value(&self) -> i32 { self.value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn new_sets_fields() {
        let s = SkillClassWithValue::new(SkillId::Block, 2);
        assert_eq!(s.get_value(), 2);
    }

    #[test]
    fn get_skill_id_returns_id() {
        let s = SkillClassWithValue::new(SkillId::Block, 0);
        assert_eq!(s.get_skill_id(), SkillId::Block);
    }
}
