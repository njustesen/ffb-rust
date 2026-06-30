use ffb_model::enums::Direction;
use ffb_model::types::FieldCoordinate;
use crate::mechanic::{Mechanic, MechanicType};
use crate::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.ThrowInMechanic.
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

    fn is_corner_throw_in(&self, _start_coordinate: FieldCoordinate) -> bool {
        false
    }

    fn interpret_throw_in_direction_roll(&self, start: FieldCoordinate, roll: i32) -> Direction {
        if start.x < 1 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::East, roll);
        }
        if start.x > 24 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::West, roll);
        }
        if start.y > 13 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::North, roll);
        }
        if start.y < 1 {
            return self.interpret_throw_in_direction_roll_with_template(Direction::South, roll);
        }
        panic!("Unable to determine throwInDirection.");
    }
}

impl ThrowInMechanic {
    pub fn new() -> Self { ThrowInMechanic }
}
