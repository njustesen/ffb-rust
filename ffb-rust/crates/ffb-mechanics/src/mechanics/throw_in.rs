use ffb_model::enums::{Direction, Rules};

/// Throw-in distance from two D6 results.
/// BB2020 adds 1 to the sum; all other editions sum the two dice directly.
pub fn throw_in_distance(die1: i32, die2: i32, rules: Rules) -> i32 {
    let base = die1 + die2;
    match rules {
        Rules::Bb2020 => base + 1,
        _ => base,
    }
}

/// Whether the coordinate is a corner square (BB2025 only).
pub fn is_corner_square(x: i32, y: i32) -> bool {
    (x < 1 || x > 24) && (y < 1 || y > 13)
}

/// Corner direction from a corner coordinate.
pub fn corner_direction(x: i32, y: i32) -> Direction {
    match (x < 1, y < 1) {
        (true,  true)  => Direction::Northwest,
        (false, true)  => Direction::Northeast,
        (true,  false) => Direction::Southwest,
        (false, false) => Direction::Southeast,
    }
}

/// Throw-in direction from a D6 roll (1–6) based on which board edge the ball left from.
/// Mirrors Java ThrowInMechanic.interpretThrowInDirectionRoll(FieldCoordinate, int).
pub fn throw_in_direction_for_roll(x: i32, y: i32, roll: i32) -> Direction {
    if x < 1  { return throw_in_direction_from_template(Direction::East,  roll); }
    if x > 24 { return throw_in_direction_from_template(Direction::West,  roll); }
    if y > 13 { return throw_in_direction_from_template(Direction::North, roll); }
    if y < 1  { return throw_in_direction_from_template(Direction::South, roll); }
    panic!("coordinate ({}, {}) is not on the board edge", x, y);
}

/// Throw-in direction from a D3 roll (1–3) for BB2025 corner squares.
pub fn corner_throw_in_direction_for_roll(corner: Direction, roll: i32) -> Direction {
    match corner {
        Direction::Northwest => match roll { 1 => Direction::East,  2 => Direction::Southeast, _ => Direction::South },
        Direction::Northeast => match roll { 1 => Direction::South, 2 => Direction::Southwest,  _ => Direction::West },
        Direction::Southwest => match roll { 1 => Direction::North, 2 => Direction::Northeast,  _ => Direction::East },
        Direction::Southeast => match roll { 1 => Direction::West,  2 => Direction::Northwest,  _ => Direction::North },
        _ => panic!("not a corner direction: {:?}", corner),
    }
}

fn throw_in_direction_from_template(template: Direction, roll: i32) -> Direction {
    match template {
        Direction::East  => match roll { 1 | 2 => Direction::Northeast, 3 | 4 => Direction::East,  _ => Direction::Southeast },
        Direction::West  => match roll { 1 | 2 => Direction::Southwest, 3 | 4 => Direction::West,  _ => Direction::Northwest },
        Direction::North => match roll { 1 | 2 => Direction::Northwest, 3 | 4 => Direction::North, _ => Direction::Northeast },
        Direction::South => match roll { 1 | 2 => Direction::Southeast, 3 | 4 => Direction::South, _ => Direction::Southwest },
        _ => panic!("not a cardinal direction template: {:?}", template),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── throw_in_distance ─────────────────────────────────────────────────────

    #[test]
    fn bb2016_sums_two_dice() {
        assert_eq!(throw_in_distance(3, 4, Rules::Bb2016), 7);
        assert_eq!(throw_in_distance(1, 1, Rules::Bb2016), 2);
        assert_eq!(throw_in_distance(6, 6, Rules::Bb2016), 12);
    }

    #[test]
    fn bb2020_adds_bonus_one() {
        assert_eq!(throw_in_distance(3, 4, Rules::Bb2020), 8);
        assert_eq!(throw_in_distance(1, 1, Rules::Bb2020), 3);
        assert_eq!(throw_in_distance(6, 6, Rules::Bb2020), 13);
    }

    #[test]
    fn bb2025_sums_two_dice_no_bonus() {
        assert_eq!(throw_in_distance(3, 4, Rules::Bb2025), 7);
        assert_eq!(throw_in_distance(1, 1, Rules::Bb2025), 2);
    }

    // ── is_corner_square ──────────────────────────────────────────────────────

    #[test]
    fn corner_squares_all_four() {
        assert!(is_corner_square(0, 0));
        assert!(is_corner_square(25, 0));
        assert!(is_corner_square(0, 14));
        assert!(is_corner_square(25, 14));
    }

    #[test]
    fn edge_not_corner() {
        assert!(!is_corner_square(5, 0));
        assert!(!is_corner_square(0, 7));
        assert!(!is_corner_square(12, 7));
    }

    // ── throw_in_direction_for_roll ───────────────────────────────────────────

    #[test]
    fn home_endzone_rolls() {
        assert_eq!(throw_in_direction_for_roll(0, 7, 1), Direction::Northeast);
        assert_eq!(throw_in_direction_for_roll(0, 7, 2), Direction::Northeast);
        assert_eq!(throw_in_direction_for_roll(0, 7, 3), Direction::East);
        assert_eq!(throw_in_direction_for_roll(0, 7, 4), Direction::East);
        assert_eq!(throw_in_direction_for_roll(0, 7, 5), Direction::Southeast);
        assert_eq!(throw_in_direction_for_roll(0, 7, 6), Direction::Southeast);
    }

    #[test]
    fn away_endzone_rolls() {
        assert_eq!(throw_in_direction_for_roll(25, 7, 1), Direction::Southwest);
        assert_eq!(throw_in_direction_for_roll(25, 7, 3), Direction::West);
        assert_eq!(throw_in_direction_for_roll(25, 7, 5), Direction::Northwest);
    }

    #[test]
    fn lower_sideline_rolls() {
        assert_eq!(throw_in_direction_for_roll(12, 14, 1), Direction::Northwest);
        assert_eq!(throw_in_direction_for_roll(12, 14, 3), Direction::North);
        assert_eq!(throw_in_direction_for_roll(12, 14, 5), Direction::Northeast);
    }

    #[test]
    fn upper_sideline_rolls() {
        assert_eq!(throw_in_direction_for_roll(12, 0, 1), Direction::Southeast);
        assert_eq!(throw_in_direction_for_roll(12, 0, 3), Direction::South);
        assert_eq!(throw_in_direction_for_roll(12, 0, 5), Direction::Southwest);
    }

    // ── corner_throw_in_direction_for_roll ────────────────────────────────────

    #[test]
    fn northwest_corner_d3_rolls() {
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northwest, 1), Direction::East);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northwest, 2), Direction::Southeast);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northwest, 3), Direction::South);
    }

    #[test]
    fn northeast_corner_d3_rolls() {
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northeast, 1), Direction::South);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northeast, 2), Direction::Southwest);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Northeast, 3), Direction::West);
    }

    #[test]
    fn southwest_corner_d3_rolls() {
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southwest, 1), Direction::North);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southwest, 2), Direction::Northeast);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southwest, 3), Direction::East);
    }

    #[test]
    fn southeast_corner_d3_rolls() {
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southeast, 1), Direction::West);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southeast, 2), Direction::Northwest);
        assert_eq!(corner_throw_in_direction_for_roll(Direction::Southeast, 3), Direction::North);
    }

    #[test]
    fn corner_direction_all_four() {
        assert_eq!(corner_direction(0, 0),   Direction::Northwest);
        assert_eq!(corner_direction(25, 0),  Direction::Northeast);
        assert_eq!(corner_direction(0, 14),  Direction::Southwest);
        assert_eq!(corner_direction(25, 14), Direction::Southeast);
    }
}
