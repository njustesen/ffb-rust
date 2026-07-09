/// 1:1 translation of Java BriberyAndCorruptionAction enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BriberyAndCorruptionAction {
    ADDED,
    USED,
    WASTED,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variants_distinct() {
        assert_ne!(BriberyAndCorruptionAction::ADDED, BriberyAndCorruptionAction::USED);
        assert_ne!(BriberyAndCorruptionAction::USED, BriberyAndCorruptionAction::WASTED);
    }

    #[test]
    fn test_copy() {
        let a = BriberyAndCorruptionAction::ADDED;
        let b = a;
        assert_eq!(a, b);
    }
}
