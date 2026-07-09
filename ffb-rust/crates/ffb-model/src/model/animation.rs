use serde::{Deserialize, Serialize};
use crate::model::animation_type::AnimationType;
use crate::enums::Direction;
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.model.Animation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Animation {
    pub animation_type: Option<AnimationType>,
    pub start_coordinate: Option<FieldCoordinate>,
    pub end_coordinate: Option<FieldCoordinate>,
    pub direction: Option<Direction>,
}

impl Animation {
    pub fn new() -> Self { Self::default() }

    pub fn with_type(mut self, t: AnimationType) -> Self {
        self.animation_type = Some(t);
        self
    }

    pub fn with_start(mut self, c: FieldCoordinate) -> Self {
        self.start_coordinate = Some(c);
        self
    }

    pub fn with_end(mut self, c: FieldCoordinate) -> Self {
        self.end_coordinate = Some(c);
        self
    }

    pub fn get_animation_type(&self) -> Option<AnimationType> { self.animation_type }
    pub fn get_direction(&self) -> Option<Direction> { self.direction }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_type() {
        assert!(Animation::new().animation_type.is_none());
    }

    #[test]
    fn with_type_sets_type() {
        let a = Animation::new().with_type(AnimationType::PASS);
        assert_eq!(a.get_animation_type(), Some(AnimationType::PASS));
    }
}
