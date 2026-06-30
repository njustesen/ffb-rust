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

#[cfg(test)]
mod tests {
    use super::*;

    // ── throw_in_distance ────────────────────────────────────────────────────

    #[test]
    fn throw_in_distance_bb2016_sums_dice() {
        assert_eq!(ThrowInCalc::throw_in_distance(3, 4, Rules::Bb2016), 7);
    }

    #[test]
    fn throw_in_distance_bb2025_sums_dice() {
        assert_eq!(ThrowInCalc::throw_in_distance(2, 5, Rules::Bb2025), 7);
    }

    #[test]
    fn throw_in_distance_bb2020_adds_1() {
        assert_eq!(ThrowInCalc::throw_in_distance(3, 4, Rules::Bb2020), 8);
    }

    // ── is_corner_square ─────────────────────────────────────────────────────

    #[test]
    fn is_corner_square_home_upper() {
        assert!(ThrowInCalc::is_corner_square(0, 0));
    }

    #[test]
    fn is_corner_square_away_lower() {
        assert!(ThrowInCalc::is_corner_square(25, 14));
    }

    #[test]
    fn is_corner_square_midfield_is_not() {
        assert!(!ThrowInCalc::is_corner_square(12, 7));
    }

    #[test]
    fn is_corner_square_endzone_midline_is_not() {
        // x<1 but y=7 is on endzone line, not corner
        assert!(!ThrowInCalc::is_corner_square(0, 7));
    }

    // ── throw_in_direction_for_roll ──────────────────────────────────────────

    #[test]
    fn throw_in_direction_home_endzone_roll_1_is_northeast() {
        // x < 1, template=EAST, roll=1 → NE
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(0, 7, 1),
            Some(Direction::Northeast)
        );
    }

    #[test]
    fn throw_in_direction_home_endzone_roll_3_is_east() {
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(0, 7, 3),
            Some(Direction::East)
        );
    }

    #[test]
    fn throw_in_direction_home_endzone_roll_5_is_southeast() {
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(0, 7, 5),
            Some(Direction::Southeast)
        );
    }

    #[test]
    fn throw_in_direction_away_endzone_roll_3_is_west() {
        // x > 24, template=WEST
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(25, 7, 3),
            Some(Direction::West)
        );
    }

    #[test]
    fn throw_in_direction_lower_sideline_roll_3_is_north() {
        // y > 13, template=NORTH
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(12, 14, 3),
            Some(Direction::North)
        );
    }

    #[test]
    fn throw_in_direction_upper_sideline_roll_3_is_south() {
        // y < 1, template=SOUTH
        assert_eq!(
            ThrowInCalc::throw_in_direction_for_roll(12, 0, 3),
            Some(Direction::South)
        );
    }

    #[test]
    fn throw_in_direction_interior_returns_none() {
        assert_eq!(ThrowInCalc::throw_in_direction_for_roll(12, 7, 4), None);
    }

    // ── corner_throw_in_direction_for_roll ───────────────────────────────────

    #[test]
    fn corner_northwest_roll_1_is_east() {
        assert_eq!(
            ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Northwest, 1),
            Some(Direction::East)
        );
    }

    #[test]
    fn corner_northwest_roll_3_is_south() {
        assert_eq!(
            ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Northwest, 3),
            Some(Direction::South)
        );
    }

    #[test]
    fn corner_southeast_roll_1_is_west() {
        assert_eq!(
            ThrowInCalc::corner_throw_in_direction_for_roll(Direction::Southeast, 1),
            Some(Direction::West)
        );
    }

    #[test]
    fn corner_non_corner_direction_returns_none() {
        assert_eq!(
            ThrowInCalc::corner_throw_in_direction_for_roll(Direction::North, 2),
            None
        );
    }

    // ── corner_direction ─────────────────────────────────────────────────────

    #[test]
    fn corner_direction_home_upper_is_northwest() {
        assert_eq!(ThrowInCalc::corner_direction(0, 0), Direction::Northwest);
    }

    #[test]
    fn corner_direction_away_upper_is_northeast() {
        assert_eq!(ThrowInCalc::corner_direction(25, 0), Direction::Northeast);
    }

    #[test]
    fn corner_direction_home_lower_is_southwest() {
        assert_eq!(ThrowInCalc::corner_direction(0, 14), Direction::Southwest);
    }

    #[test]
    fn corner_direction_away_lower_is_southeast() {
        assert_eq!(ThrowInCalc::corner_direction(25, 14), Direction::Southeast);
    }
}
