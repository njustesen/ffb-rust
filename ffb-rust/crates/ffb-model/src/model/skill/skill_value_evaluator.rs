/// 1:1 translation of com.fumbbl.ffb.model.skill.SkillValueEvaluator.
///
/// Java models this as an interface with static constants (DEFAULT, MODIFIER, ROLL).
/// Rust models it as an enum-like type with associated behaviour.
use std::collections::HashSet;

/// Mirrors `SkillValueEvaluator` variants from Java.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillValueEvaluator {
    /// Java `SkillValueEvaluator.DEFAULT` — no numeric value annotation.
    Default,
    /// Java `SkillValueEvaluator.MODIFIER` — uses max of available values, formats as "+N".
    Modifier,
    /// Java `SkillValueEvaluator.ROLL` — uses min of available values, formats as "N+".
    Roll,
}

impl SkillValueEvaluator {
    /// Java `intValue(Set<String> tempValues)`.
    /// Returns `None` for `Default`, max integer for `Modifier`, min integer for `Roll`.
    pub fn int_value(&self, temp_values: &HashSet<String>) -> Option<i32> {
        let parsed: Vec<i32> = temp_values
            .iter()
            .filter_map(|v| v.parse::<i32>().ok())
            .collect();
        match self {
            SkillValueEvaluator::Default => None,
            SkillValueEvaluator::Modifier => parsed.iter().copied().max(),
            SkillValueEvaluator::Roll => parsed.iter().copied().min(),
        }
    }
}

impl Default for SkillValueEvaluator {
    fn default() -> Self {
        SkillValueEvaluator::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_of(vals: &[&str]) -> HashSet<String> {
        vals.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn default_evaluator_returns_none() {
        assert_eq!(SkillValueEvaluator::Default.int_value(&set_of(&["1", "2"])), None);
    }

    #[test]
    fn modifier_returns_max() {
        assert_eq!(SkillValueEvaluator::Modifier.int_value(&set_of(&["1", "3", "2"])), Some(3));
    }

    #[test]
    fn roll_returns_min() {
        assert_eq!(SkillValueEvaluator::Roll.int_value(&set_of(&["4", "2", "3"])), Some(2));
    }

    #[test]
    fn empty_set_modifier_returns_none() {
        assert_eq!(SkillValueEvaluator::Modifier.int_value(&set_of(&[])), None);
    }

    #[test]
    fn empty_set_roll_returns_none() {
        assert_eq!(SkillValueEvaluator::Roll.int_value(&set_of(&[])), None);
    }

    #[test]
    fn non_numeric_strings_are_ignored() {
        let vals = set_of(&["foo", "bar", "5"]);
        assert_eq!(SkillValueEvaluator::Modifier.int_value(&vals), Some(5));
    }

    #[test]
    fn default_impl_is_default_variant() {
        assert_eq!(SkillValueEvaluator::default(), SkillValueEvaluator::Default);
    }
}
