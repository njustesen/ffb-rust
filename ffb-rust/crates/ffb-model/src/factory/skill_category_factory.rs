use crate::enums::SkillCategory;

/// 1:1 translation of com.fumbbl.ffb.factory.SkillCategoryFactory.
pub struct SkillCategoryFactory;

impl Default for SkillCategoryFactory {
    fn default() -> Self { SkillCategoryFactory }
}

impl SkillCategoryFactory {
    /// Java: `forName` -- over `SkillCategory.values()` in declaration order, `equalsIgnoreCase`
    /// with the name or the alt name (only MUTATION has one: "Mutations", so MUTATIONS itself is
    /// never returned); null for an empty name.
    pub fn for_name(&self, name: &str) -> Option<SkillCategory> {
        use crate::util::java_json::java_equals_ignore_case;
        if name.is_empty() {
            return None;
        }
        const VALUES: [(SkillCategory, Option<&str>); 11] = [
            (SkillCategory::General, None), (SkillCategory::Agility, None), (SkillCategory::Passing, None),
            (SkillCategory::Strength, None), (SkillCategory::Mutation, Some("Mutations")),
            (SkillCategory::Mutations, None), (SkillCategory::Extraordinary, None),
            (SkillCategory::StatIncrease, None), (SkillCategory::StatDecrease, None),
            (SkillCategory::Trait, None), (SkillCategory::Devious, None),
        ];
        VALUES
            .iter()
            .find(|(c, alt)| {
                java_equals_ignore_case(name, c.name()) || alt.is_some_and(|a| java_equals_ignore_case(name, a))
            })
            .map(|(c, _)| *c)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_category() {
        assert_eq!(SkillCategoryFactory::default().for_name("General"), Some(SkillCategory::General));
        assert_eq!(SkillCategoryFactory::default().for_name("Agility"), Some(SkillCategory::Agility));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SkillCategoryFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_strength_returns_some() {
        let f = SkillCategoryFactory::default();
        assert!(f.for_name("Strength").is_some());
    }

    #[test]
    fn for_name_matches_java_ignore_case_and_alt_name() {
        assert_eq!(SkillCategoryFactory.for_name("stat increase"), Some(SkillCategory::StatIncrease));
        // MUTATION's alt name wins over MUTATIONS itself (declaration order).
        assert_eq!(SkillCategoryFactory.for_name("Mutations"), Some(SkillCategory::Mutation));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(SkillCategoryFactory.for_name("").is_none());
    }
}
