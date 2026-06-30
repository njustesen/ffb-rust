use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::enums::{PlayerState, SkillId};
use crate::model::player::PlayerId;

/// 1:1 translation of com.fumbbl.ffb.model.TargetSelectionState.Status.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetSelectionStatus {
    STARTED,
    CANCELED,
    SELECTED,
    SKIPPED,
    FAILED,
}

/// 1:1 translation of com.fumbbl.ffb.model.TargetSelectionState.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSelectionState {
    pub status: TargetSelectionStatus,
    pub selected_player_id: Option<PlayerId>,
    pub committed: bool,
    pub old_acting_player_state: Option<PlayerState>,
    pub used_skills: HashSet<SkillId>,
}

impl Default for TargetSelectionState {
    fn default() -> Self {
        TargetSelectionState {
            status: TargetSelectionStatus::STARTED,
            selected_player_id: None,
            committed: false,
            old_acting_player_state: None,
            used_skills: HashSet::new(),
        }
    }
}

impl TargetSelectionState {
    pub fn new(selected_player_id: impl Into<PlayerId>) -> Self {
        TargetSelectionState { selected_player_id: Some(selected_player_id.into()), ..Default::default() }
    }

    pub fn get_selected_player_id(&self) -> Option<&PlayerId> { self.selected_player_id.as_ref() }
    pub fn get_status(&self) -> TargetSelectionStatus { self.status }

    pub fn cancel(&mut self) -> &mut Self { self.status = TargetSelectionStatus::CANCELED; self }
    pub fn select(&mut self) -> &mut Self { self.status = TargetSelectionStatus::SELECTED; self }
    pub fn skip(&mut self) -> &mut Self { self.status = TargetSelectionStatus::SKIPPED; self }
    pub fn failed(&mut self) -> &mut Self { self.status = TargetSelectionStatus::FAILED; self }

    pub fn is_started(&self) -> bool { self.status == TargetSelectionStatus::STARTED }
    pub fn is_canceled(&self) -> bool { self.status == TargetSelectionStatus::CANCELED }
    pub fn is_selected(&self) -> bool { self.status == TargetSelectionStatus::SELECTED }
    pub fn is_skipped(&self) -> bool { self.status == TargetSelectionStatus::SKIPPED }
    pub fn is_failed(&self) -> bool { self.status == TargetSelectionStatus::FAILED }

    pub fn is_committed(&self) -> bool { self.committed }
    pub fn commit(&mut self) {
        if !self.committed {
            self.committed = true;
        }
    }

    pub fn get_old_acting_player_state(&self) -> Option<PlayerState> { self.old_acting_player_state }
    pub fn set_old_acting_player_state(&mut self, state: Option<PlayerState>) {
        self.old_acting_player_state = state;
    }

    pub fn get_used_skills(&self) -> &HashSet<SkillId> { &self.used_skills }
    pub fn add_used_skill(&mut self, skill: SkillId) { self.used_skills.insert(skill); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_status_is_started() {
        assert_eq!(TargetSelectionState::default().get_status(), TargetSelectionStatus::STARTED);
    }

    #[test]
    fn cancel_sets_canceled() {
        let mut s = TargetSelectionState::default();
        s.cancel();
        assert!(s.is_canceled());
    }

    #[test]
    fn commit_is_idempotent() {
        let mut s = TargetSelectionState::default();
        s.commit();
        s.commit();
        assert!(s.is_committed());
    }

    #[test]
    fn serde_round_trip() {
        let mut s = TargetSelectionState::new("p1");
        s.select();
        s.commit();
        s.add_used_skill(SkillId::Dodge);
        let json = serde_json::to_string(&s).unwrap();
        let back: TargetSelectionState = serde_json::from_str(&json).unwrap();
        assert!(back.is_selected());
        assert!(back.is_committed());
        assert!(back.used_skills.contains(&SkillId::Dodge));
    }
}
