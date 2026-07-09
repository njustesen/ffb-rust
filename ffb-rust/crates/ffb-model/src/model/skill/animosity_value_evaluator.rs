/// 1:1 translation of com.fumbbl.ffb.model.skill.AnimosityValueEvaluator.
///
/// Java: `interface AnimosityValueEvaluator extends SkillValueEvaluator { String allValue(); }`
/// Rust: a trait extending the behaviour of `SkillValueEvaluator`.
use crate::model::skill::skill_value_evaluator::SkillValueEvaluator;
use std::collections::HashSet;

/// Extends `SkillValueEvaluator` with an `all_value()` sentinel string.
pub trait AnimosityValueEvaluator {
    /// Java `allValue()` — returns the sentinel string meaning "all races".
    fn all_value(&self) -> &str;
    /// Delegates to `SkillValueEvaluator::int_value`.
    fn int_value_for_animosity(&self, temp_values: &HashSet<String>) -> Option<i32> {
        SkillValueEvaluator::Default.int_value(temp_values)
    }
}

/// Default implementation for Animosity — `all_value()` returns `"all"`.
pub struct DefaultAnimosityValueEvaluator;

impl AnimosityValueEvaluator for DefaultAnimosityValueEvaluator {
    fn all_value(&self) -> &str {
        "all"
    }
}

impl Default for DefaultAnimosityValueEvaluator {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_value_returns_all() {
        assert_eq!(DefaultAnimosityValueEvaluator.all_value(), "all");
    }

    #[test]
    fn int_value_returns_none_for_empty() {
        let vals = HashSet::new();
        assert_eq!(DefaultAnimosityValueEvaluator.int_value_for_animosity(&vals), None);
    }
}
