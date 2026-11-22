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

    /// Java: `TargetSelectionState.toJsonValue()`; skill names from the game's ruleset
    /// `SkillFactory` (`Err` for a skill that ruleset has no class for).
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as j;
        let mut o = j::JsonObject::new();
        o.insert("playerId".into(), j::string_or_null(self.selected_player_id.as_deref()));
        o.insert("targetSelectionStatus".into(), serde_json::Value::from(status_name(self.status)));
        o.insert("targetSelectionStatusIsCommitted".into(), serde_json::Value::Bool(self.committed));
        o.insert("playerStateOld".into(), j::player_state_or_null(self.old_acting_player_state.map(|s| s.0 as i32)));
        // Java iterates a HashSet (the canonical form sorts this array).
        let mut used = Vec::with_capacity(self.used_skills.len());
        for id in &self.used_skills {
            let name = SkillFactory::name_in(rules, *id)
                .ok_or_else(|| format!("TargetSelectionState: {id:?} is not a {} skill", rules.name()))?;
            used.push(serde_json::Value::from(name));
        }
        o.insert("usedSkills".into(), serde_json::Value::Array(used));
        Ok(serde_json::Value::Object(o))
    }

    /// Java: `new TargetSelectionState().initFrom(source, jsonValue)`: the legacy `blitzStatus`
    /// key when `targetSelectionStatus` is not provided; `Status.valueOf` throws on an unknown
    /// name (`Err`); the defined-only fields keep the constructor default otherwise. An unknown
    /// skill name (Java adds `null` to the set) is `Err`.
    pub fn init_from(rules: crate::enums::Rules, json: &serde_json::Value) -> Result<Self, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let mut state = TargetSelectionState::default();
        state.selected_player_id = j::get_string(o, "playerId")?;
        let mut status = j::get_string(o, "targetSelectionStatus")?;
        if status.as_deref().map_or(true, str::is_empty) {
            status = j::get_string(o, "blitzStatus")?;
        }
        if let Some(name) = status.filter(|s| !s.is_empty()) {
            state.status = status_value_of(&name)
                .ok_or_else(|| format!("TargetSelectionState: Status.valueOf({name:?}) throws"))?;
        }
        if j::is_defined_in(o, "targetSelectionStatusIsCommitted") {
            state.committed = j::get_boolean(o, "targetSelectionStatusIsCommitted")?;
        }
        if j::is_defined_in(o, "playerStateOld") {
            state.old_acting_player_state = j::get_player_state(o, "playerStateOld")?;
        }
        if j::is_defined_in(o, "usedSkills") {
            let used = j::get_array(o, "usedSkills")?.ok_or("TargetSelectionState: usedSkills null (NPE)")?;
            for element in used {
                let name = element.as_str().ok_or_else(|| format!("usedSkills: element {element}"))?;
                let id = SkillFactory::for_name_in(rules, name).ok_or_else(|| {
                    format!("TargetSelectionState: skill {name:?} unknown to {} (Java null)", rules.name())
                })?;
                state.used_skills.insert(id);
            }
        }
        Ok(state)
    }
}

/// Java `Status.name()`.
fn status_name(status: TargetSelectionStatus) -> &'static str {
    match status {
        TargetSelectionStatus::STARTED => "STARTED",
        TargetSelectionStatus::CANCELED => "CANCELED",
        TargetSelectionStatus::SELECTED => "SELECTED",
        TargetSelectionStatus::SKIPPED => "SKIPPED",
        TargetSelectionStatus::FAILED => "FAILED",
    }
}

/// Java `Status.valueOf(name)` (`None` where Java throws).
fn status_value_of(name: &str) -> Option<TargetSelectionStatus> {
    use TargetSelectionStatus::*;
    [STARTED, CANCELED, SELECTED, SKIPPED, FAILED].into_iter().find(|s| status_name(*s) == name)
}

#[cfg(test)]
mod tests {
    use super::*;

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
