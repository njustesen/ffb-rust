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
}
