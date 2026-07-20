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

// Tests mirror ffb-java/ffb-server/src/test/java/com/fumbbl/ffb/server/util/ScatterCalcTest.java 1:1
#[cfg(test)]
mod tests {
    use super::*;

    // ── direction_for_roll ────────────────────────────────────────────────────

    #[test]
    fn direction_for_roll_all_faces() {
        for (roll, expected) in [
            (1, Direction::North),
            (2, Direction::Northeast),
            (3, Direction::East),
            (4, Direction::Southeast),
            (5, Direction::South),
            (6, Direction::Southwest),
            (7, Direction::West),
            (8, Direction::Northwest),
        ] {
            assert_eq!(Some(expected), ScatterCalc::direction_for_roll(roll), "roll {roll}");
        }
    }

    #[test]
    fn direction_for_roll_out_of_range_returns_null() {
        assert_eq!(None, ScatterCalc::direction_for_roll(0));
        assert_eq!(None, ScatterCalc::direction_for_roll(9));
    }

    // ── scatter_coordinate ────────────────────────────────────────────────────

    #[test]
    fn scatter_north_decreases_y() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::North, 1);
        assert_eq!(FieldCoordinate::new(10, 9), result);
    }

    #[test]
    fn scatter_south_increases_y() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::South, 1);
        assert_eq!(FieldCoordinate::new(10, 11), result);
    }

    #[test]
    fn scatter_east_increases_x() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::East, 1);
        assert_eq!(FieldCoordinate::new(11, 10), result);
    }

    #[test]
    fn scatter_west_decreases_x() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::West, 1);
        assert_eq!(FieldCoordinate::new(9, 10), result);
    }

    #[test]
    fn scatter_northeast_increases_both() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::Northeast, 1);
        assert_eq!(FieldCoordinate::new(11, 9), result);
    }

    #[test]
    fn scatter_southeast_increases_x_increases_y() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::Southeast, 1);
        assert_eq!(FieldCoordinate::new(11, 11), result);
    }

    #[test]
    fn scatter_southwest_decreases_x_increases_y() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::Southwest, 1);
        assert_eq!(FieldCoordinate::new(9, 11), result);
    }

    #[test]
    fn scatter_northwest_decreases_both() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 10), Direction::Northwest, 1);
        assert_eq!(FieldCoordinate::new(9, 9), result);
    }

    #[test]
    fn scatter_distance_two_doubles_offset() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(5, 5), Direction::Northeast, 2);
        assert_eq!(FieldCoordinate::new(7, 3), result);
    }

    #[test]
    fn scatter_distance_zero_returns_start() {
        let start = FieldCoordinate::new(7, 7);
        let result = ScatterCalc::scatter_coordinate(start, Direction::South, 0);
        assert_eq!(start, result);
    }

    #[test]
    fn scatter_southwest_distance_three() {
        let result = ScatterCalc::scatter_coordinate(FieldCoordinate::new(10, 5), Direction::Southwest, 3);
        // dx=-3, dy=+3
        assert_eq!(FieldCoordinate::new(7, 8), result);
    }
}
