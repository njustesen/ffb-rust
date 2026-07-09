/// 1:1 translation of com.fumbbl.ffb.model.skill.ISkillValueEvaluator (Java interface).
pub trait SkillValueEvaluator {
    fn evaluate(&self, value: i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Identity;
    impl SkillValueEvaluator for Identity {
        fn evaluate(&self, value: i32) -> i32 { value }
    }

    #[test]
    fn identity_returns_same() {
        assert_eq!(Identity.evaluate(3), 3);
    }

    #[test]
    fn identity_zero() {
        assert_eq!(Identity.evaluate(0), 0);
    }
}
