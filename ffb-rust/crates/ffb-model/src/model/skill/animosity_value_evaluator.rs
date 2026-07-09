use super::skill_value_evaluator::SkillValueEvaluator;

/// 1:1 translation of com.fumbbl.ffb.model.skill.AnimosityValueEvaluator.
pub struct AnimosityValueEvaluator;

impl SkillValueEvaluator for AnimosityValueEvaluator {
    fn evaluate(&self, value: i32) -> i32 { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::skill_value_evaluator::SkillValueEvaluator;

    #[test]
    fn evaluate_returns_value() {
        assert_eq!(AnimosityValueEvaluator.evaluate(5), 5);
    }

    #[test]
    fn evaluate_zero() {
        assert_eq!(AnimosityValueEvaluator.evaluate(0), 0);
    }
}
