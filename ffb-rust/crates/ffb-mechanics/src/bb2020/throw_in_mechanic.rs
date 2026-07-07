use ffb_model::enums::Direction;
use ffb_model::types::FieldCoordinate;
use crate::mechanic::{Mechanic, MechanicType};
use crate::throw_in_mechanic::ThrowInMechanic as ThrowInMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.ThrowInMechanic.
pub struct ThrowInMechanic;

impl Default for ThrowInMechanic {
    fn default() -> Self { ThrowInMechanic }
}

impl Mechanic for ThrowInMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::THROW_IN }
}

impl ThrowInMechanicTrait for ThrowInMechanic {
    fn distance(&self, distance_roll: &[i32]) -> i32 {
        distance_roll[0] + distance_roll[1] + 1
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
    fn distance_sums_two_dice_plus_one() {
        // bb2020 adds 1 to the total
        assert_eq!(ThrowInMechanic.distance(&[3, 4]), 8);
    }

    #[test]
    fn direction_from_south_edge_roll_1_is_southeast() {
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(12, 0), 1);
        assert_eq!(dir, Direction::Southeast);
    }

    #[test]
    fn direction_from_north_edge_roll_4_is_north() {
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(12, 14), 4);
        assert_eq!(dir, Direction::North);
    }

    #[test]
    fn distance_minimum_dice() {
        assert_eq!(ThrowInMechanic.distance(&[1, 1]), 3);
    }

    #[test]
    fn distance_maximum_dice() {
        assert_eq!(ThrowInMechanic.distance(&[6, 6]), 13);
    }

    #[test]
    fn is_corner_throw_in_always_false() {
        // BB2020 ThrowInMechanic always returns false for corner throw-in
        assert!(!ThrowInMechanic.is_corner_throw_in(FieldCoordinate::new(0, 0)));
        assert!(!ThrowInMechanic.is_corner_throw_in(FieldCoordinate::new(24, 14)));
    }

    #[test]
    fn direction_from_east_edge() {
        // x > 24 → start template is West
        let dir = ThrowInMechanic.interpret_throw_in_direction_roll(FieldCoordinate::new(25, 7), 4);
        assert_eq!(dir, Direction::West);
    }

    #[test]
    fn mechanic_type_is_throw_in() {
        assert_eq!(crate::mechanic::Mechanic::get_type(&ThrowInMechanic), MechanicType::THROW_IN);
    }
}
