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
    fn serde_round_trip() {
        for v in [InjuryModification::ARMOUR, InjuryModification::INJURY, InjuryModification::NONE] {
            let s = serde_json::to_string(&v).unwrap();
            let back: InjuryModification = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back);
        }
    }

}
