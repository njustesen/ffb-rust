use ffb_model::enums::Direction;
use ffb_model::types::FieldCoordinate;
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.ThrowInMechanic.
pub trait ThrowInMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::THROW_IN }

    fn distance(&self, distance_roll: &[i32]) -> i32;
    fn is_corner_throw_in(&self, start_coordinate: FieldCoordinate) -> bool;
    fn interpret_throw_in_direction_roll(&self, start: FieldCoordinate, roll: i32) -> Direction;

    /// 1:1 translation of interpretThrowInDirectionRoll(Direction, int) (concrete in Java).
    fn interpret_throw_in_direction_roll_with_template(&self, template_direction: Direction, roll: i32) -> Direction {
        match template_direction {
            Direction::East => match roll {
                1 | 2 => Direction::Northeast,
                3 | 4 => Direction::East,
                5 | 6 => Direction::Southeast,
                _ => panic!("Unable to determine throwInDirection."),
            },
            Direction::West => match roll {
                1 | 2 => Direction::Southwest,
                3 | 4 => Direction::West,
                5 | 6 => Direction::Northwest,
                _ => panic!("Unable to determine throwInDirection."),
            },
            Direction::North => match roll {
                1 | 2 => Direction::Northwest,
                3 | 4 => Direction::North,
                5 | 6 => Direction::Northeast,
                _ => panic!("Unable to determine throwInDirection."),
            },
            Direction::South => match roll {
                1 | 2 => Direction::Southeast,
                3 | 4 => Direction::South,
                5 | 6 => Direction::Southwest,
                _ => panic!("Unable to determine throwInDirection."),
            },
            _ => panic!("Unable to determine throwInDirection."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;
    use crate::mechanic::{Mechanic, MechanicType};

    struct TestThrowIn;
    impl Mechanic for TestThrowIn {
        fn get_type(&self) -> MechanicType { MechanicType::THROW_IN }
    }
    impl ThrowInMechanic for TestThrowIn {
        fn distance(&self, _: &[i32]) -> i32 { 0 }
        fn is_corner_throw_in(&self, _: FieldCoordinate) -> bool { false }
        fn interpret_throw_in_direction_roll(&self, _: FieldCoordinate, _: i32) -> Direction { Direction::East }
    }

    #[test]
    fn east_roll_1_2_is_northeast() {
        let m = TestThrowIn;
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::East, 1), Direction::Northeast);
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::East, 2), Direction::Northeast);
    }

    #[test]
    fn west_roll_3_4_is_west() {
        let m = TestThrowIn;
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::West, 3), Direction::West);
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::West, 4), Direction::West);
    }

    #[test]
    fn north_roll_5_6_is_northeast() {
        let m = TestThrowIn;
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::North, 5), Direction::Northeast);
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::North, 6), Direction::Northeast);
    }

    #[test]
    fn south_roll_1_2_is_southeast() {
        let m = TestThrowIn;
        assert_eq!(m.interpret_throw_in_direction_roll_with_template(Direction::South, 1), Direction::Southeast);
    }
}
