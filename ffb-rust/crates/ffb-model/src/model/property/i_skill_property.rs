/// 1:1 translation of com.fumbbl.ffb.model.property.ISkillProperty.
///
/// In Java this is an interface. Every ISkillProperty has a name string that
/// uniquely identifies it. Two ISkillProperty instances are equal iff their
/// names are equal.
pub trait ISkillProperty {
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::property::NamedProperty;

    #[test]
    fn named_property_implements_i_skill_property() {
        let p = NamedProperty::new("canLeap");
        assert_eq!(p.name(), "canLeap");
    }

    #[test]
    fn i_skill_property_name_non_empty() {
        let p = NamedProperty::new("canAvoidFallingDown");
        assert!(!p.name().is_empty());
    }
}
