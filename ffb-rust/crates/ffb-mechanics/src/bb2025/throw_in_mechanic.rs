use ffb_model::enums::Direction;
use ffb_model::types::FieldCoordinate;
use crate::mechanic::{Mechanic, MechanicType};
use crate::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.ThrowInMechanic.
pub struct ThrowInMechanic;

impl Default for ThrowInMechanic {
    fn default() -> Self { ThrowInMechanic }
}

impl Mechanic for ThrowInMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::THROW_IN }
}

impl ThrowInMechanicTrait for ThrowInMechanic {
    fn distance(&self, distance_roll: &[i32]) -> i32 {
        distance_roll[0] + distance_roll[1]
    }

    fn is_corner_throw_in(&self, start_coordinate: FieldCoordinate) -> bool {
        (start_coordinate.x < 1 || start_coordinate.x > 24)
            && (start_coordinate.y < 1 || start_coordinate.y > 13)
    }

    fn interpret_throw_in_direction_roll(&self, start: FieldCoordinate, roll: i32) -> Direction {
        if start.x < 1 && start.y < 1 {
            return self.interpret_corner_throw_in_direction_roll(Direction::Northwest, roll);
        }
        if start.x > 24 && start.y < 1 {
            return self.interpret_corner_throw_in_direction_roll(Direction::Northeast, roll);
        }
        if start.x < 1 && start.y > 13 {
            return self.interpret_corner_throw_in_direction_roll(Direction::Southwest, roll);
        }
        if start.x > 24 && start.y > 13 {
            return self.interpret_corner_throw_in_direction_roll(Direction::Southeast, roll);
        }
        // Endzone Home Team
        if start.x < 1 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::East, roll);
        }
        // Endzone Away Team
        if start.x > 24 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::West, roll);
        }
        // Lower Sideline
        if start.y > 13 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::North, roll);
        }
        // Upper Sideline
        if start.y < 1 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::South, roll);
        }
        panic!("Unable to determine throwInDirection.");
    }
}

impl ThrowInMechanic {
    pub fn new() -> Self { ThrowInMechanic }

    fn interpret_corner_throw_in_direction_roll(&self, corner_direction: Direction, roll: i32) -> Direction {
        match corner_direction {
            Direction::Northwest => match roll {
                1 => Direction::East,
                2 => Direction::Southeast,
                3 => Direction::South,
                _ => panic!("Unable to determine cornerThrowInDirection."),
            },
            Direction::Northeast => match roll {
                1 => Direction::South,
                2 => Direction::Southwest,
                3 => Direction::West,
                _ => panic!("Unable to determine cornerThrowInDirection."),
            },
            Direction::Southwest => match roll {
                1 => Direction::North,
                2 => Direction::Northeast,
                3 => Direction::East,
                _ => panic!("Unable to determine cornerThrowInDirection."),
            },
            Direction::Southeast => match roll {
                1 => Direction::West,
                2 => Direction::Northwest,
                3 => Direction::North,
                _ => panic!("Unable to determine cornerThrowInDirection."),
            },
            _ => panic!("Unable to determine cornerThrowInDirection."),
        }
    }
}
