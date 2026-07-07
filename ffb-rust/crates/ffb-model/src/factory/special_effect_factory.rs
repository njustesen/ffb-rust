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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_effect() {
        let f = SpecialEffectFactory::default();
        assert!(f.for_name("LIGHTNING").is_some() || f.for_name("lightning").is_some() || f.for_name("Lightning").is_some());
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SpecialEffectFactory::default().for_name("NOT_VALID"), None);
    }

    #[test]
    fn for_name_fireball_returns_some() {
        let f = SpecialEffectFactory::default();
        assert!(f.for_name("fireball").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = SpecialEffectFactory::default();
        f.initialize();
    }
    #[test]
    fn for_name_empty_string_returns_none() {
        assert!(SpecialEffectFactory.for_name("").is_none());
    }
}
