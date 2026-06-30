use crate::model::SpecialEffect;

/// 1:1 translation of com.fumbbl.ffb.factory.SpecialEffectFactory.
pub struct SpecialEffectFactory;

impl Default for SpecialEffectFactory {
    fn default() -> Self { SpecialEffectFactory }
}

impl SpecialEffectFactory {
    pub fn for_name(&self, name: &str) -> Option<SpecialEffect> {
        SpecialEffect::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
