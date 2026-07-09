use serde::{Deserialize, Serialize};
use crate::enums::SkillId;

/// 1:1 translation of com.fumbbl.ffb.model.property.CancelSkillProperty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelSkillProperty {
    pub skill_id: SkillId,
}

impl CancelSkillProperty {
    pub fn new(skill_id: SkillId) -> Self { Self { skill_id } }
    pub fn get_skill_id(&self) -> SkillId { self.skill_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn new_sets_skill_id() {
        let p = CancelSkillProperty::new(SkillId::Block);
        assert_eq!(p.get_skill_id(), SkillId::Block);
    }

    #[test]
    fn skill_id_is_block() {
        let p = CancelSkillProperty::new(SkillId::Block);
        assert_eq!(p.skill_id, SkillId::Block);
    }
}
