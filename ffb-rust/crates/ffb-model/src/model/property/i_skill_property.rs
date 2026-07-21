/// 1:1 translation of com.fumbbl.ffb.model.property.ISkillProperty.
///
/// In Java this is an interface. Every ISkillProperty has a name string that
/// uniquely identifies it. Two ISkillProperty instances are equal iff their
/// names are equal.
pub trait ISkillProperty {
    fn name(&self) -> &str;
}
