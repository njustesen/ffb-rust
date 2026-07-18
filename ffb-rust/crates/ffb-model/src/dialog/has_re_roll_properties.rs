use crate::enums::ReRollProperty;

/// 1:1 translation of com.fumbbl.ffb.HasReRollProperties.
pub trait HasReRollProperties {
    fn has_property(&self, property: ReRollProperty) -> bool;
}
