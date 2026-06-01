use ffb_model::enums::Direction;

/// Compute new (x, y) after scattering from (x, y) in `direction` for `distance` squares.
/// Does not clamp or validate board bounds.
/// Mirrors Java UtilServerCatchScatterThrowIn.findScatterCoordinate().
pub fn scatter_coordinate(x: i32, y: i32, direction: Direction, distance: i32) -> (i32, i32) {
    (x + direction.dx() as i32 * distance, y + direction.dy() as i32 * distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Direction;

    #[test]
    fn scatter_north_decreases_y() {
        assert_eq!(scatter_coordinate(10, 10, Direction::North, 1), (10, 9));
    }

    #[test]
    fn scatter_south_increases_y() {
        assert_eq!(scatter_coordinate(10, 10, Direction::South, 1), (10, 11));
    }

    #[test]
    fn scatter_east_increases_x() {
        assert_eq!(scatter_coordinate(10, 10, Direction::East, 1), (11, 10));
    }

    #[test]
    fn scatter_west_decreases_x() {
        assert_eq!(scatter_coordinate(10, 10, Direction::West, 1), (9, 10));
    }

    #[test]
    fn scatter_northeast() {
        assert_eq!(scatter_coordinate(10, 10, Direction::Northeast, 1), (11, 9));
    }

    #[test]
    fn scatter_southeast() {
        assert_eq!(scatter_coordinate(10, 10, Direction::Southeast, 1), (11, 11));
    }

    #[test]
    fn scatter_southwest() {
        assert_eq!(scatter_coordinate(10, 10, Direction::Southwest, 1), (9, 11));
    }

    #[test]
    fn scatter_northwest() {
        assert_eq!(scatter_coordinate(10, 10, Direction::Northwest, 1), (9, 9));
    }

    #[test]
    fn scatter_distance_two_doubles_offset() {
        assert_eq!(scatter_coordinate(5, 5, Direction::Northeast, 2), (7, 3));
    }

    #[test]
    fn scatter_distance_zero_stays_in_place() {
        assert_eq!(scatter_coordinate(7, 7, Direction::South, 0), (7, 7));
    }

    #[test]
    fn all_directions_produce_unit_offset_at_distance_one() {
        for dir in Direction::all() {
            let (nx, ny) = scatter_coordinate(0, 0, *dir, 1);
            let dx = nx.abs();
            let dy = ny.abs();
            assert!(dx <= 1 && dy <= 1, "direction {:?} produced non-unit offset ({}, {})", dir, nx, ny);
            assert!(nx != 0 || ny != 0, "direction {:?} produced zero offset", dir);
        }
    }
}
