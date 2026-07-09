use crate::enums::ReRollProperty;

/// 1:1 translation of com.fumbbl.ffb.model.IHasReRollProperties (Java interface).
pub trait HasReRollProperties {
    fn has_re_roll_property(&self, prop: ReRollProperty) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ReRollProperty;

    struct AlwaysTrue;
    impl HasReRollProperties for AlwaysTrue {
        fn has_re_roll_property(&self, _: ReRollProperty) -> bool { true }
    }

    struct AlwaysFalse;
    impl HasReRollProperties for AlwaysFalse {
        fn has_re_roll_property(&self, _: ReRollProperty) -> bool { false }
    }

    #[test]
    fn always_true_impl() {
        assert!(AlwaysTrue.has_re_roll_property(ReRollProperty::Trr));
    }

    #[test]
    fn always_false_impl() {
        assert!(!AlwaysFalse.has_re_roll_property(ReRollProperty::Trr));
    }
}
