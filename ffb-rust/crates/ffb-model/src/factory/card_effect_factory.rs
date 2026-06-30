use crate::enums::CardEffect;

/// 1:1 translation of com.fumbbl.ffb.factory.CardEffectFactory.
pub struct CardEffectFactory;

impl Default for CardEffectFactory {
    fn default() -> Self { CardEffectFactory }
}

impl CardEffectFactory {
    pub fn for_name(&self, name: &str) -> Option<CardEffect> {
        CardEffect::from_name(name)
    }

    pub fn initialize(&mut self) {}
}
