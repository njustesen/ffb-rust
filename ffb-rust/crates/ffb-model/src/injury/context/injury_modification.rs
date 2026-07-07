use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.injury.context.InjuryModification.
/// Records which phase of the injury roll was modified by a skill.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InjuryModification {
    ARMOUR,
    INJURY,
    #[default]
    NONE,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(InjuryModification::default(), InjuryModification::NONE);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(InjuryModification::ARMOUR, InjuryModification::INJURY);
        assert_ne!(InjuryModification::ARMOUR, InjuryModification::NONE);
    }

    #[test]
    fn copy_semantics() {
        let a = InjuryModification::ARMOUR;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn serde_round_trip() {
        for v in [InjuryModification::ARMOUR, InjuryModification::INJURY, InjuryModification::NONE] {
            let s = serde_json::to_string(&v).unwrap();
            let back: InjuryModification = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn injury_variant_is_not_default() {
        assert_ne!(InjuryModification::INJURY, InjuryModification::default());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", InjuryModification::default()).is_empty());
    }

}
