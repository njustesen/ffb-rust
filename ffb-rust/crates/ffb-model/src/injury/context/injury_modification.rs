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
}
