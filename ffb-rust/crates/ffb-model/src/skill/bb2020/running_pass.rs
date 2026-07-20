/// 1:1 translation of com.fumbbl.ffb.skill.bb2020::RunningPass.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct RunningPass {
    pub base: Skill,
}

impl RunningPass {
    pub fn new() -> Self {
        let base = Skill::new("Running Pass", SkillCategory::Passing);
        Self { base }
    }
}

impl Default for RunningPass {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for RunningPass {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::SkillId;

    #[test]
    fn name_is_running_pass() {
        assert_eq!(RunningPass::new().get_name(), "Running Pass");
    }

    #[test]
    fn category_is_passing() {
        assert_eq!(RunningPass::new().get_category(), SkillCategory::Passing);
    }

    #[test]
    fn has_can_move_after_quick_pass_property() {
        assert!(SkillId::RunningPass.properties().contains(&"canMoveAfterQuickPass"));
    }
}
