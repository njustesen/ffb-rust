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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::throw_in_mechanic::ThrowInMechanic as ThrowInTrait;

    #[test]
    fn distance_sums_two_dice() {
        assert_eq!(ThrowInMechanic.distance(&[3, 4]), 7);
    }

    #[test]
    fn is_corner_throw_in_always_false() {
        assert!(!ThrowInMechanic.is_corner_throw_in(FieldCoordinate::new(0, 0)));
    }

    #[test]
    fn direction_from_west_edge_roll_3_is_east() {
        // x < 1 → template East; roll 3 or 4 → East
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(0, 7), 3);
        assert_eq!(dir, Direction::East);
    }

    #[test]
    fn direction_from_east_edge_roll_1_is_southwest() {
        // x > 24 → template West; roll 1 or 2 → Southwest
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(25, 7), 1);
        assert_eq!(dir, Direction::Southwest);
    }

    #[test]
    fn direction_from_north_edge_roll_1_is_northwest() {
        // y > 13 → template North; roll 1 or 2 → Northwest
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(12, 14), 1);
        assert_eq!(dir, Direction::Northwest);
    }
}
