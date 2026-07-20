// 1:1 translation of com.fumbbl.ffb.server.util.ThrowInCalc
//
// Pure throw-in mechanics: direction and distance from D6 rolls.
//
// Board bounds: x=0 is home endzone, x=25 is away endzone; y=0 is upper sideline, y=14 is lower sideline.

use ffb_model::enums::{Direction, Rules};

pub struct ThrowInCalc;

impl ThrowInCalc {
    pub fn new() -> Self {
        Self
    }

    /// Throw-in distance from two D6 results.
    /// BB2020 adds 1; all other editions sum the two dice directly.
    pub fn throw_in_distance(die1: i32, die2: i32, rules: Rules) -> i32 {
        let base = die1 + die2;
        if rules == Rules::Bb2020 { base + 1 } else { base }
    }

    /// Whether the coordinate is a corner square (BB2025 only).
    /// Corners exist at the intersections of both endzones and both sidelines.
    pub fn is_corner_square(x: i32, y: i32) -> bool {
        (x < 1 || x > 24) && (y < 1 || y > 13)
    }

    /// Throw-in direction from a D6 roll (1–6) based on which edge the ball left from.
    /// Returns one of three directions: the two diagonals flanking the inward direction, or straight in.
    ///
    /// Coordinate conventions:
    /// - x < 1  → home endzone, ball goes EAST (inward)
    /// - x > 24 → away endzone, ball goes WEST (inward)
    /// - y > 13 → lower sideline, ball goes NORTH (inward)
    /// - y < 1  → upper sideline, ball goes SOUTH (inward)
    ///
    /// Returns `None` if the coordinate is not on the board edge.
    pub fn throw_in_direction_for_roll(x: i32, y: i32, roll: i32) -> Option<Direction> {
        if x < 1 {
            Some(Self::throw_in_direction_from_template(Direction::East, roll))
        } else if x > 24 {
            Some(Self::throw_in_direction_from_template(Direction::West, roll))
        } else if y > 13 {
            Some(Self::throw_in_direction_from_template(Direction::North, roll))
        } else if y < 1 {
            Some(Self::throw_in_direction_from_template(Direction::South, roll))
        } else {
            None
        }
    }

    /// Throw-in direction from a D3 roll (1–3) for BB2025 corner squares.
    /// The corner direction identifies which corner (e.g. Northwest = x<1, y<1).
    /// Returns `None` if `corner_direction` is not a corner direction.
    pub fn corner_throw_in_direction_for_roll(
        corner_direction: Direction,
        roll: i32,
    ) -> Option<Direction> {
        match corner_direction {
            Direction::Northwest => match roll {
                1 => Some(Direction::East),
                2 => Some(Direction::Southeast),
                _ => Some(Direction::South),
            },
            Direction::Northeast => match roll {
                1 => Some(Direction::South),
                2 => Some(Direction::Southwest),
                _ => Some(Direction::West),
            },
            Direction::Southwest => match roll {
                1 => Some(Direction::North),
                2 => Some(Direction::Northeast),
                _ => Some(Direction::East),
            },
            Direction::Southeast => match roll {
                1 => Some(Direction::West),
                2 => Some(Direction::Northwest),
                _ => Some(Direction::North),
            },
            _ => None,
        }
    }

    /// Which corner direction applies to the given corner coordinate (BB2025).
    pub fn corner_direction(x: i32, y: i32) -> Direction {
        let west = x < 1;
        let north = y < 1;
        if west && north {
            Direction::Northwest
        } else if !west && north {
            Direction::Northeast
        } else if west {
            Direction::Southwest
        } else {
            Direction::Southeast
        }
    }

    // Mirrors ThrowInMechanic.interpretThrowInDirectionRoll(Direction, int)
    fn throw_in_direction_from_template(template: Direction, roll: i32) -> Direction {
        match template {
            Direction::East => match roll {
                1 | 2 => Direction::Northeast,
                3 | 4 => Direction::East,
                _ => Direction::Southeast,
            },
            Direction::West => match roll {
                1 | 2 => Direction::Southwest,
                3 | 4 => Direction::West,
                _ => Direction::Northwest,
            },
            Direction::North => match roll {
                1 | 2 => Direction::Northwest,
                3 | 4 => Direction::North,
                _ => Direction::Northeast,
            },
            Direction::South => match roll {
                1 | 2 => Direction::Southeast,
                3 | 4 => Direction::South,
                _ => Direction::Southwest,
            },
            // Non-cardinal templates are not used; treat as straight-in with identity
            other => other,
        }
    }
}

impl Default for ThrowInCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Test mirror of com.fumbbl.ffb.server.util.ThrowInCalcTest (1:1).
// Java @ParameterizedTest CsvSource rows become one Rust test fn looping the same rows.
// Note: Java signals invalid input with IllegalArgumentException; the Rust mirror returns None.
#[cfg(test)]
mod tests {
    use super::*;

    // ── throw_in_distance ────────────────────────────────────────────────────

    #[test]
    fn distance_bb2016_sums_two_dice() {
        assert_eq!(ThrowInCalc::throw_in_distance(3, 4, Rules::Bb2016), 7);
        assert_eq!(ThrowInCalc::throw_in_distance(1, 1, Rules::Bb2016), 2);
        assert_eq!(ThrowInCalc::throw_in_distance(6, 6, Rules::Bb2016), 12);
    }

    #[test]
    fn distance_bb2020_adds_bonus_one() {
        assert_eq!(ThrowInCalc::throw_in_distance(3, 4, Rules::Bb2020), 8);
        assert_eq!(ThrowInCalc::throw_in_distance(1, 1, Rules::Bb2020), 3);
        assert_eq!(ThrowInCalc::throw_in_distance(6, 6, Rules::Bb2020), 13);
    }

    #[test]
    fn distance_bb2025_sums_two_dice_no_bonus_like_bb2016() {
        assert_eq!(ThrowInCalc::throw_in_distance(3, 4, Rules::Bb2025), 7);
        assert_eq!(ThrowInCalc::throw_in_distance(1, 1, Rules::Bb2025), 2);
    }

    // ── is_corner_square ─────────────────────────────────────────────────────

    #[test]
    fn is_corner_square_all_four_corners() {
        assert!(ThrowInCalc::is_corner_square(0, 0));
        assert!(ThrowInCalc::is_corner_square(25, 0));
        assert!(ThrowInCalc::is_corner_square(0, 14));
        assert!(ThrowInCalc::is_corner_square(25, 14));
    }

    #[test]
    fn is_corner_square_edge_not_corner() {
        assert!(!ThrowInCalc::is_corner_square(5, 0)); // upper sideline, not corner
        assert!(!ThrowInCalc::is_corner_square(0, 7)); // home endzone, not corner
        assert!(!ThrowInCalc::is_corner_square(12, 7)); // field
    }

    // ── throw_in_direction_for_roll ──────────────────────────────────────────

    #[test]
    fn home_endzone_directions_for_rolls() {
        let rows = [
            (1, Direction::Northeast),
            (2, Direction::Northeast),
            (3, Direction::East),
            (4, Direction::East),
            (5, Direction::Southeast),
            (6, Direction::Southeast),
        ];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::throw_in_direction_for_roll(0, 7, roll),
                Some(expected),
                "home-endzone roll {roll}"
            );
        }
    }

    #[test]
    fn away_endzone_directions_for_rolls() {
        let rows = [
            (1, Direction::Southwest),
            (2, Direction::Southwest),
            (3, Direction::West),
            (4, Direction::West),
            (5, Direction::Northwest),
            (6, Direction::Northwest),
        ];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::throw_in_direction_for_roll(25, 7, roll),
                Some(expected),
                "away-endzone roll {roll}"
            );
        }
    }

    #[test]
    fn lower_sideline_directions_for_rolls() {
        let rows = [
            (1, Direction::Northwest),
            (2, Direction::Northwest),
            (3, Direction::North),
            (4, Direction::North),
            (5, Direction::Northeast),
            (6, Direction::Northeast),
        ];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::throw_in_direction_for_roll(12, 14, roll),
                Some(expected),
                "lower-sideline roll {roll}"
            );
        }
    }

    #[test]
    fn upper_sideline_directions_for_rolls() {
        let rows = [
            (1, Direction::Southeast),
            (2, Direction::Southeast),
            (3, Direction::South),
            (4, Direction::South),
            (5, Direction::Southwest),
            (6, Direction::Southwest),
        ];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::throw_in_direction_for_roll(12, 0, roll),
                Some(expected),
                "upper-sideline roll {roll}"
            );
        }
    }

    #[test]
    fn throw_in_direction_interior_coordinate_is_invalid() {
        // (12,7) is not on any board edge — the Rust mirror returns None
        // (Java signals this with an IllegalArgumentException)
        assert_eq!(ThrowInCalc::throw_in_direction_for_roll(12, 7, 4), None);
    }

    // ── corner_throw_in_direction_for_roll ───────────────────────────────────

    #[test]
    fn northwest_corner() {
        let rows = [(1, Direction::East), (2, Direction::Southeast), (3, Direction::South)];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Northwest, roll),
                Some(expected),
                "NW-corner D3={roll}"
            );
        }
    }

    #[test]
    fn northeast_corner() {
        let rows = [(1, Direction::South), (2, Direction::Southwest), (3, Direction::West)];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Northeast, roll),
                Some(expected),
                "NE-corner D3={roll}"
            );
        }
    }

    #[test]
    fn southwest_corner() {
        let rows = [(1, Direction::North), (2, Direction::Northeast), (3, Direction::East)];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Southwest, roll),
                Some(expected),
                "SW-corner D3={roll}"
            );
        }
    }

    #[test]
    fn southeast_corner() {
        let rows = [(1, Direction::West), (2, Direction::Northwest), (3, Direction::North)];
        for (roll, expected) in rows {
            assert_eq!(
                ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Southeast, roll),
                Some(expected),
                "SE-corner D3={roll}"
            );
        }
    }

    #[test]
    fn corner_throw_in_direction_non_corner_direction_is_invalid() {
        // NORTH is not a corner direction — the Rust mirror returns None
        // (Java signals this with an IllegalArgumentException)
        assert_eq!(
            ThrowInCalc::corner_throw_in_direction_for_roll(Direction::North, 2),
            None
        );
    }

    // ── corner_direction ─────────────────────────────────────────────────────

    #[test]
    fn corner_direction_all_four_corners() {
        assert_eq!(ThrowInCalc::corner_direction(0, 0), Direction::Northwest);
        assert_eq!(ThrowInCalc::corner_direction(25, 0), Direction::Northeast);
        assert_eq!(ThrowInCalc::corner_direction(0, 14), Direction::Southwest);
        assert_eq!(ThrowInCalc::corner_direction(25, 14), Direction::Southeast);
    }
}
