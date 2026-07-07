/// 1:1 translation of `com.fumbbl.ffb.server.ActionStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatus {
    SkillChoiceYes,
    SkillChoiceNo,
    Success,
    WaitingForReRoll,
    WaitingForSkillUse,
    WaitForActionChange,
    Failure,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_is_not_failure() {
        assert_ne!(ActionStatus::Success, ActionStatus::Failure);
    }

    #[test]
    fn skill_choice_variants_are_distinct() {
        assert_ne!(ActionStatus::SkillChoiceYes, ActionStatus::SkillChoiceNo);
    }

    #[test]
    fn waiting_variants_are_distinct() {
        assert_ne!(ActionStatus::WaitingForReRoll, ActionStatus::WaitingForSkillUse);
        assert_ne!(ActionStatus::WaitingForSkillUse, ActionStatus::WaitForActionChange);
    }

    #[test]
    fn all_variants_are_pairwise_distinct() {
        let variants = [
            ActionStatus::SkillChoiceYes,
            ActionStatus::SkillChoiceNo,
            ActionStatus::Success,
            ActionStatus::WaitingForReRoll,
            ActionStatus::WaitingForSkillUse,
            ActionStatus::WaitForActionChange,
            ActionStatus::Failure,
        ];
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    #[test]
    fn copy_semantics_preserved() {
        let a = ActionStatus::Success;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn clone_equals_original() {
        let a = ActionStatus::WaitingForReRoll;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format_contains_variant_name() {
        let s = format!("{:?}", ActionStatus::Failure);
        assert!(s.contains("Failure"));
        let s2 = format!("{:?}", ActionStatus::SkillChoiceYes);
        assert!(s2.contains("SkillChoiceYes"));
    }

    #[test]
    fn wait_for_action_change_is_distinct_from_failure() {
        assert_ne!(ActionStatus::WaitForActionChange, ActionStatus::Failure);
    }
}
