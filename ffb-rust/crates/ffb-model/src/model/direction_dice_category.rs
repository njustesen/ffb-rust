use crate::enums::{Direction, DiceCategoryKind};

/// 1:1 translation of com.fumbbl.ffb.model.DirectionDiceCategory.
pub struct DirectionDiceCategory;

impl DirectionDiceCategory {
    pub fn kind() -> DiceCategoryKind { DiceCategoryKind::Direction }
    pub fn roll_to_direction(roll: i32) -> Option<Direction> { Direction::for_roll(roll) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Direction;

    #[test]
    fn kind_is_direction() {
        assert_eq!(DirectionDiceCategory::kind(), DiceCategoryKind::Direction);
    }

    #[test]
    fn roll_1_is_north() {
        assert_eq!(DirectionDiceCategory::roll_to_direction(1), Some(Direction::North));
    }
}
