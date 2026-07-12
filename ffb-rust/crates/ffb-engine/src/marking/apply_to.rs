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

    /// Java: `Enum.name()` — the variant's declared constant name, used verbatim as the
    /// `IJsonOption.APPLY_TO` wire value in `AutoMarkingRecord.toJsonValue()`
    /// (`applyTo.name()`).
    pub fn name(&self) -> &'static str {
        match self {
            ApplyTo::Own => "OWN",
            ApplyTo::Opponent => "OPPONENT",
            ApplyTo::Both => "BOTH",
        }
    }

    /// Java: `ApplyTo.valueOf(String)` — inverse of [`Self::name`], used in
    /// `AutoMarkingRecord.initFrom` (`ApplyTo.valueOf(IJsonOption.APPLY_TO.getFrom(...))`).
    /// Java throws `IllegalArgumentException` on an unknown name; this returns `None`.
    pub fn value_of(name: &str) -> Option<ApplyTo> {
        match name {
            "OWN" => Some(ApplyTo::Own),
            "OPPONENT" => Some(ApplyTo::Opponent),
            "BOTH" => Some(ApplyTo::Both),
            _ => None,
        }
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

    #[test]
    fn name_matches_java_constant_names() {
        assert_eq!(ApplyTo::Own.name(), "OWN");
        assert_eq!(ApplyTo::Opponent.name(), "OPPONENT");
        assert_eq!(ApplyTo::Both.name(), "BOTH");
    }

    #[test]
    fn value_of_round_trips_name() {
        for variant in [ApplyTo::Own, ApplyTo::Opponent, ApplyTo::Both] {
            assert_eq!(ApplyTo::value_of(variant.name()), Some(variant));
        }
    }

    #[test]
    fn value_of_unknown_is_none() {
        assert_eq!(ApplyTo::value_of("SIDEWAYS"), None);
        assert_eq!(ApplyTo::value_of("own"), None); // case-sensitive, like Java valueOf
    }
}
