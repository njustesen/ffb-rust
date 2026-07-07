/// 1:1 translation of com.fumbbl.ffb.util.RaiseType.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RaiseType {
    ZOMBIE,
    ROTTER,
    THRALL,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_distinct() {
        assert_ne!(RaiseType::ZOMBIE, RaiseType::ROTTER);
        assert_ne!(RaiseType::ROTTER, RaiseType::THRALL);
    }

    #[test]
    fn three_variants() {
        let all = [RaiseType::ZOMBIE, RaiseType::ROTTER, RaiseType::THRALL];
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn copy_semantics() {
        let a = RaiseType::ZOMBIE;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn all_variants_are_distinct_from_each_other() {
        assert_ne!(RaiseType::ZOMBIE, RaiseType::THRALL);
        assert_ne!(RaiseType::ROTTER, RaiseType::ZOMBIE);
        assert_ne!(RaiseType::ROTTER, RaiseType::THRALL);
    }

    #[test]
    fn equality_is_reflexive() {
        assert_eq!(RaiseType::ZOMBIE, RaiseType::ZOMBIE);
        assert_eq!(RaiseType::ROTTER, RaiseType::ROTTER);
        assert_eq!(RaiseType::THRALL, RaiseType::THRALL);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", RaiseType::ZOMBIE).is_empty());
    }

}
