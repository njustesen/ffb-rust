use crate::enums::ReRollProperty;

/// 1:1 translation of com.fumbbl.ffb.model.IHasReRollProperties (Java interface).
pub trait HasReRollProperties {
    fn has_re_roll_property(&self, prop: ReRollProperty) -> bool;
}
