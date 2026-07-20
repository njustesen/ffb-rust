use ffb_model::enums::Direction;

/// Compute new (x, y) after scattering from (x, y) in `direction` for `distance` squares.
/// Does not clamp or validate board bounds.
/// Mirrors Java UtilServerCatchScatterThrowIn.findScatterCoordinate().
pub fn scatter_coordinate(x: i32, y: i32, direction: Direction, distance: i32) -> (i32, i32) {
    (x + direction.dx() as i32 * distance, y + direction.dy() as i32 * distance)
}

// Java-derived cases (ScatterCalcTest) live in the 1:1 mirror module
// ffb-engine/src/util/scatter_calc.rs; the former per-direction tests here
// duplicated them (same inputs and expectations) and were removed. Only the
// property test the mirror does not cover remains.
#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Direction;

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
