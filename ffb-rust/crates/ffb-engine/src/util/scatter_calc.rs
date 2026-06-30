// 1:1 translation of com.fumbbl.ffb.server.util.ScatterCalc
use ffb_model::enums::Direction;
use ffb_model::types::FieldCoordinate;

pub struct ScatterCalc;

impl ScatterCalc {
    pub fn new() -> Self {
        Self
    }

    /// Map a D8 roll (1–8) to a scatter direction.
    /// Mirrors DirectionFactory.forRoll(): 1=North, 2=Northeast, ..., 8=Northwest.
    /// Returns None for out-of-range rolls.
    pub fn direction_for_roll(roll: i32) -> Option<Direction> {
        Direction::for_roll(roll)
    }

    /// Compute the coordinate after scattering from `start` in `direction` for `distance` squares.
    /// Does not clamp or validate board bounds.
    pub fn scatter_coordinate(start: FieldCoordinate, direction: Direction, distance: i32) -> FieldCoordinate {
        let dx = direction.dx() as i32 * distance;
        let dy = direction.dy() as i32 * distance;
        start.add(dx, dy)
    }
}

impl Default for ScatterCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_1_is_north() {
        assert_eq!(ScatterCalc::direction_for_roll(1), Some(Direction::North));
    }

    #[test]
    fn roll_5_is_south() {
        assert_eq!(ScatterCalc::direction_for_roll(5), Some(Direction::South));
    }

    #[test]
    fn roll_8_is_northwest() {
        assert_eq!(ScatterCalc::direction_for_roll(8), Some(Direction::Northwest));
    }

    #[test]
    fn roll_out_of_range_is_none() {
        assert_eq!(ScatterCalc::direction_for_roll(0), None);
        assert_eq!(ScatterCalc::direction_for_roll(9), None);
    }

    #[test]
    fn all_eight_rolls_produce_direction() {
        for roll in 1..=8 {
            assert!(ScatterCalc::direction_for_roll(roll).is_some(), "roll {} should give direction", roll);
        }
    }

    #[test]
    fn scatter_north_moves_up() {
        let start = FieldCoordinate::new(10, 10);
        let result = ScatterCalc::scatter_coordinate(start, Direction::North, 1);
        assert_eq!(result, FieldCoordinate::new(10, 9));
    }

    #[test]
    fn scatter_south_moves_down() {
        let start = FieldCoordinate::new(10, 10);
        let result = ScatterCalc::scatter_coordinate(start, Direction::South, 1);
        assert_eq!(result, FieldCoordinate::new(10, 11));
    }

    #[test]
    fn scatter_east_moves_right() {
        let start = FieldCoordinate::new(10, 10);
        let result = ScatterCalc::scatter_coordinate(start, Direction::East, 1);
        assert_eq!(result, FieldCoordinate::new(11, 10));
    }

    #[test]
    fn scatter_northeast_moves_diagonally() {
        let start = FieldCoordinate::new(10, 10);
        let result = ScatterCalc::scatter_coordinate(start, Direction::Northeast, 1);
        assert_eq!(result, FieldCoordinate::new(11, 9));
    }

    #[test]
    fn scatter_distance_two() {
        let start = FieldCoordinate::new(5, 5);
        let result = ScatterCalc::scatter_coordinate(start, Direction::South, 2);
        assert_eq!(result, FieldCoordinate::new(5, 7));
    }

    #[test]
    fn scatter_southwest_distance_three() {
        let start = FieldCoordinate::new(10, 5);
        let result = ScatterCalc::scatter_coordinate(start, Direction::Southwest, 3);
        // dx=-3, dy=+3
        assert_eq!(result, FieldCoordinate::new(7, 8));
    }
}
