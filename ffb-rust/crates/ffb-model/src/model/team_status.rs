use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.TeamStatus.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamStatus {
    NEW,
    ACTIVE,
    PENDING_APPROVAL,
    BLOCKED,
    RETIRED,
    WAITING_FOR_OPPONENT,
    SKILL_ROLLS_PENDING,
}

impl TeamStatus {
    pub fn get_id(self) -> i32 {
        match self {
            TeamStatus::NEW => 0,
            TeamStatus::ACTIVE => 1,
            TeamStatus::PENDING_APPROVAL => 2,
            TeamStatus::BLOCKED => 3,
            TeamStatus::RETIRED => 4,
            TeamStatus::WAITING_FOR_OPPONENT => 5,
            TeamStatus::SKILL_ROLLS_PENDING => 6,
        }
    }

    pub fn get_name(self) -> &'static str {
        match self {
            TeamStatus::NEW => "New",
            TeamStatus::ACTIVE => "Active",
            TeamStatus::PENDING_APPROVAL => "Pending Approval",
            TeamStatus::BLOCKED => "Blocked",
            TeamStatus::RETIRED => "Retired",
            TeamStatus::WAITING_FOR_OPPONENT => "Waiting for Opponent",
            TeamStatus::SKILL_ROLLS_PENDING => "Skill Rolls Pending",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_has_id_1() {
        assert_eq!(TeamStatus::ACTIVE.get_id(), 1);
    }
    #[test]
    fn active_name_is_active() {
        assert_eq!(TeamStatus::ACTIVE.get_name(), "Active");
    }
    #[test]
    fn all_ids_are_unique() {
        let ids: Vec<i32> = [
            TeamStatus::NEW, TeamStatus::ACTIVE, TeamStatus::PENDING_APPROVAL,
            TeamStatus::BLOCKED, TeamStatus::RETIRED, TeamStatus::WAITING_FOR_OPPONENT,
            TeamStatus::SKILL_ROLLS_PENDING,
        ].iter().map(|s| s.get_id()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn ids_are_sequential_from_zero() {
        let mut ids: Vec<i32> = [
            TeamStatus::NEW, TeamStatus::ACTIVE, TeamStatus::PENDING_APPROVAL,
            TeamStatus::BLOCKED, TeamStatus::RETIRED, TeamStatus::WAITING_FOR_OPPONENT,
            TeamStatus::SKILL_ROLLS_PENDING,
        ].iter().map(|s| s.get_id()).collect();
        ids.sort();
        assert_eq!(ids, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn all_names_are_non_empty_and_distinct() {
        let names: Vec<&str> = [
            TeamStatus::NEW, TeamStatus::ACTIVE, TeamStatus::PENDING_APPROVAL,
            TeamStatus::BLOCKED, TeamStatus::RETIRED, TeamStatus::WAITING_FOR_OPPONENT,
            TeamStatus::SKILL_ROLLS_PENDING,
        ].iter().map(|s| s.get_name()).collect();
        for name in &names {
            assert!(!name.is_empty());
        }
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", TeamStatus::NEW).is_empty());
    }

}
