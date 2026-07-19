/// 1:1 translation of com.fumbbl.ffb.skill.bb2025::ProjectileVomit.
use crate::model::skill::skill::Skill;
use crate::enums::{DeclareCondition, SkillCategory};

pub struct ProjectileVomit {
    pub base: Skill,
}

impl ProjectileVomit {
    pub fn new() -> Self {
        let mut base = Skill::new("Projectile Vomit", SkillCategory::Trait);
        base.declare_condition = DeclareCondition::Standing;
        Self { base }
    }
}

impl Default for ProjectileVomit {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for ProjectileVomit {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(ProjectileVomit::new().get_name(), "Projectile Vomit");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(ProjectileVomit::new().get_category(), SkillCategory::Trait);
    }

    #[test]
    fn declare_condition_is_standing() {
        // Java constructor calls setDeclareCondition(DeclareCondition.STANDING).
        assert_eq!(ProjectileVomit::new().get_declare_condition(), DeclareCondition::Standing);
    }
}
