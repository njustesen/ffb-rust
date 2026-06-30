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
