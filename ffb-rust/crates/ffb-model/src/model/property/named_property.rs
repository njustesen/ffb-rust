use super::i_skill_property::ISkillProperty;

/// 1:1 translation of com.fumbbl.ffb.model.property.NamedProperty.
///
/// A concrete ISkillProperty identified purely by its name string. Two
/// NamedProperty values with the same name are considered equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedProperty {
    name: &'static str,
}

impl NamedProperty {
    pub const fn new(name: &'static str) -> Self {
        NamedProperty { name }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl ISkillProperty for NamedProperty {
    fn name(&self) -> &str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_constructor_argument() {
        let p = NamedProperty::new("canLeap");
        assert_eq!(p.name(), "canLeap");
    }

    #[test]
    fn equality_by_name() {
        assert_eq!(NamedProperty::new("canLeap"), NamedProperty::new("canLeap"));
        assert_ne!(NamedProperty::new("canLeap"), NamedProperty::new("canRun"));
    }

    #[test]
    fn const_construction() {
        const P: NamedProperty = NamedProperty::new("canLeap");
        assert_eq!(P.name(), "canLeap");
    }

    #[test]
    fn i_skill_property_trait_name_matches_field() {
        let p = NamedProperty::new("blockSkill");
        // ISkillProperty::name() must return the same value as the inherent name()
        let trait_obj: &dyn ISkillProperty = &p;
        assert_eq!(trait_obj.name(), "blockSkill");
    }

    #[test]
    fn hash_consistent_with_equality() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NamedProperty::new("canLeap"));
        // Inserting the same name again must not grow the set
        set.insert(NamedProperty::new("canLeap"));
        assert_eq!(set.len(), 1);
        // A different name must produce a distinct entry
        set.insert(NamedProperty::new("canDive"));
        assert_eq!(set.len(), 2);
    }
}
