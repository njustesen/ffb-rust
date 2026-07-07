use crate::model::AnimationType;

/// 1:1 translation of com.fumbbl.ffb.factory.AnimationTypeFactory.
pub struct AnimationTypeFactory;

impl Default for AnimationTypeFactory {
    fn default() -> Self { AnimationTypeFactory }
}

impl AnimationTypeFactory {
    pub fn for_name(&self, name: &str) -> Option<AnimationType> {
        AnimationType::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_variant() {
        let f = AnimationTypeFactory::default();
        assert_eq!(f.for_name("pass"), Some(AnimationType::PASS));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        let f = AnimationTypeFactory::default();
        assert_eq!(f.for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = AnimationTypeFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        let f = AnimationTypeFactory::default();
        assert_eq!(f.for_name("kick"), Some(AnimationType::KICK));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        let f = AnimationTypeFactory::default();
        assert_eq!(f.for_name(""), None);
    }
}
