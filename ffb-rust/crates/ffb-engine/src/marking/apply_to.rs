/// 1:1 translation of `com.fumbbl.ffb.server.marking.ApplyTo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyTo {
    Own,
    Opponent,
    Both,
}

impl ApplyTo {
    pub fn applies_to_own(&self) -> bool {
        matches!(self, ApplyTo::Own | ApplyTo::Both)
    }

    pub fn applies_to_opponent(&self) -> bool {
        matches!(self, ApplyTo::Opponent | ApplyTo::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_applies_to_own_only() {
        assert!(ApplyTo::Own.applies_to_own());
        assert!(!ApplyTo::Own.applies_to_opponent());
    }

    #[test]
    fn opponent_applies_to_opponent_only() {
        assert!(!ApplyTo::Opponent.applies_to_own());
        assert!(ApplyTo::Opponent.applies_to_opponent());
    }

    #[test]
    fn both_applies_to_both() {
        assert!(ApplyTo::Both.applies_to_own());
        assert!(ApplyTo::Both.applies_to_opponent());
    }

    #[test]
    fn all_variants_are_distinct() {
        assert_ne!(ApplyTo::Own, ApplyTo::Opponent);
        assert_ne!(ApplyTo::Own, ApplyTo::Both);
        assert_ne!(ApplyTo::Opponent, ApplyTo::Both);
    }

    #[test]
    fn copy_semantics_preserved() {
        let a = ApplyTo::Both;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn clone_equals_original() {
        let a = ApplyTo::Opponent;
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn debug_format_contains_variant_name() {
        assert!(format!("{:?}", ApplyTo::Own).contains("Own"));
        assert!(format!("{:?}", ApplyTo::Opponent).contains("Opponent"));
        assert!(format!("{:?}", ApplyTo::Both).contains("Both"));
    }

    #[test]
    fn own_eq_self() {
        assert_eq!(ApplyTo::Own, ApplyTo::Own);
    }

    #[test]
    fn both_is_copy() {
        let a = ApplyTo::Both;
        let _b = a;
        let _c = a; // copy semantics: can use after "move"
    }
}
