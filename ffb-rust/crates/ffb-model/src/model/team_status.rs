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
