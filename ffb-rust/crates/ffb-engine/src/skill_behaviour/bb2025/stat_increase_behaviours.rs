//! 1:1 translations of the three BB2025 stat-increase behaviours, each a load-time
//! `PlayerModifier` (see `registry::apply_player_modifiers_from_behaviours`, BACKLOG §M 2b(h)):
//! - `skillbehaviour.bb2025.AgilityIncreaseBehaviour`
//! - `skillbehaviour.bb2025.PassingIncreaseBehaviour`
//! - `skillbehaviour.bb2025.StrengthIncreaseBehaviour`
//! Their lambdas are identical, line for line, to the BB2020 classes (AG max(1, pos-2, ag-1); PA
//! "<= 0 -> 6" else max(1, pos-2, pa-1); ST min(8, pos+2, st+1)), so the modifiers are shared.
use ffb_model::enums::SkillId;
use ffb_model::model::PlayerModifier;

use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::bb2020::stat_increase_behaviours::{
    AgilityIncreaseModifier, PassingIncreaseModifier, StrengthIncreaseModifier,
};
use crate::skill_behaviour::registry::SkillRegistry;

fn register(registry: &mut SkillRegistry, id: SkillId, m: Box<dyn PlayerModifier>) {
    let mut sb = SbContainer::new();
    sb.register_player_modifier(m);
    registry.register(id, sb);
}

pub struct AgilityIncreaseBehaviour;
impl AgilityIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::AgilityIncrease, Box::new(AgilityIncreaseModifier));
    }
}

pub struct PassingIncreaseBehaviour;
impl PassingIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::PassingIncrease, Box::new(PassingIncreaseModifier));
    }
}

pub struct StrengthIncreaseBehaviour;
impl StrengthIncreaseBehaviour {
    pub fn register_into(registry: &mut SkillRegistry) {
        register(registry, SkillId::StrengthIncrease, Box::new(StrengthIncreaseModifier));
    }
}
