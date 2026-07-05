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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_effect() {
        assert_eq!(CardEffectFactory::default().for_name("Distracted"), Some(CardEffect::Distracted));
        assert_eq!(CardEffectFactory::default().for_name("Poisoned"), Some(CardEffect::Poisoned));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(CardEffectFactory::default().for_name("invalid"), None);
    }
}
