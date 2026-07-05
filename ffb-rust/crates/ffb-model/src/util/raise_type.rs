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
}
