/// 1:1 translation of com.fumbbl.ffb.skill.bb2016::SecretWeapon.
use crate::model::skill::skill::Skill;
use crate::enums::SkillCategory;

pub struct SecretWeapon {
    pub base: Skill,
}

impl SecretWeapon {
    pub fn new() -> Self {
        let base = Skill::new("Secret Weapon", SkillCategory::Extraordinary);
        Self { base }
    }
}

impl Default for SecretWeapon {
    fn default() -> Self { Self::new() }
}

impl std::ops::Deref for SecretWeapon {
    type Target = Skill;
    fn deref(&self) -> &Self::Target { &self.base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(SecretWeapon::new().get_name(), "Secret Weapon");
    }

    #[test]
    fn category_is_correct() {
        assert_eq!(SecretWeapon::new().get_category(), SkillCategory::Extraordinary);
    }
}
