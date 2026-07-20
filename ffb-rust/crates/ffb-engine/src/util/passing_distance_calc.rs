// 1:1 translation of com.fumbbl.ffb.server.util.PassingDistanceCalc
use ffb_model::enums::PassingDistance;

pub struct PassingDistanceCalc;

/// Passing distance table, same as BB2020/BB2025 PassMechanic.
/// Indexed: TABLE[deltaY][deltaX].
/// None entries mean the delta is out of range or invalid.
static TABLE: [[Option<PassingDistance>; 14]; 14] = build_table();

const fn char_to_distance(c: u8) -> Option<PassingDistance> {
    match c {
        b'Q' => Some(PassingDistance::QuickPass),
        b'S' => Some(PassingDistance::ShortPass),
        b'L' => Some(PassingDistance::LongPass),
        b'B' => Some(PassingDistance::LongBomb),
        _ => None, // 'T', ' ', or unknown
    }
}

const fn build_table() -> [[Option<PassingDistance>; 14]; 14] {
    // Row strings from BB2020 PassMechanic.throwingRangeTable():
    // Q=QuickPass, S=ShortPass, L=LongPass, B=LongBomb, T/space=null
    // Each entry is separated by a space, so index = dx * 2
    const ROWS: [&[u8]; 14] = [
        b"T Q Q Q S S S L L L L B B B",
        b"Q Q Q Q S S S L L L L B B B",
        b"Q Q Q S S S S L L L L B B B",
        b"Q Q S S S S S L L L B B B  ",
        b"S S S S S S L L L L B B B  ",
        b"S S S S S L L L L B B B    ",
        b"S S S S L L L L L B B B    ",
        b"L L L L L L L L B B B      ",
        b"L L L L L L L B B B B      ",
        b"L L L L L B B B B B        ",
        b"L L L B B B B B B          ",
        b"B B B B B B B              ",
        b"B B B B B                  ",
        b"B B B                      ",
    ];

    let mut table = [[None; 14]; 14];
    let mut dy = 0usize;
    while dy < 14 {
        let row = ROWS[dy];
        let mut dx = 0usize;
        while dx < 14 {
            let idx = dx * 2;
            if idx < row.len() {
                table[dy][dx] = char_to_distance(row[idx]);
            } else {
                table[dy][dx] = None;
            }
            dx += 1;
        }
        dy += 1;
    }
    table
}

impl PassingDistanceCalc {
    pub fn new() -> Self {
        Self
    }

    /// Returns the passing distance for a throw with the given absolute coordinate deltas.
    /// Returns None if the distance is out of range (too far or same square).
    pub fn for_deltas(delta_x: i32, delta_y: i32) -> Option<PassingDistance> {
        if delta_x < 0 || delta_y < 0 || delta_x >= 14 || delta_y >= 14 {
            return None;
        }
        TABLE[delta_y as usize][delta_x as usize]
    }

    /// Returns the passing distance for a throw from (from_x, from_y) to (to_x, to_y).
    /// Returns None if the distance is out of range or from==to.
    pub fn for_coordinates(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Option<PassingDistance> {
        Self::for_deltas((to_x - from_x).abs(), (to_y - from_y).abs())
    }
}

impl Default for PassingDistanceCalc {
    fn default() -> Self {
        Self::new()
    }
}

// Test mirror of com.fumbbl.ffb.server.util.PassingDistanceCalcTest (1:1).
// Java @ParameterizedTest CsvSource rows become one Rust test fn looping the same rows.
#[cfg(test)]
mod tests {
    use super::*;

    // ── Own square ───────────────────────────────────────────────────────────

    #[test]
    fn same_square_returns_null() {
        assert_eq!(PassingDistanceCalc::for_deltas(0, 0), None);
    }

    // ── Quick Pass ───────────────────────────────────────────────────────────

    #[test]
    fn quick_pass() {
        let rows = [
            (1, 0), // adjacent horizontally
            (0, 1), // adjacent vertically
            (1, 1), // diagonal
            (2, 0), // two squares horizontal
            (2, 1), // row 1
            (2, 2), // row 2
            (1, 2), // row 2
            (3, 0), // row 0
            (3, 1), // row 1
        ];
        for (dx, dy) in rows {
            assert_eq!(
                PassingDistanceCalc::for_deltas(dx, dy),
                Some(PassingDistance::QuickPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Short Pass ───────────────────────────────────────────────────────────

    #[test]
    fn short_pass() {
        let rows = [
            (4, 0), // row 0
            (5, 0), // row 0
            (6, 0), // row 0
            (3, 2), // row 2
            (4, 2), // row 2
            (0, 4), // dy=4
            (1, 4), // row 4
        ];
        for (dx, dy) in rows {
            assert_eq!(
                PassingDistanceCalc::for_deltas(dx, dy),
                Some(PassingDistance::ShortPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Long Pass ────────────────────────────────────────────────────────────

    #[test]
    fn long_pass() {
        let rows = [
            (7, 0),  // row 0
            (8, 0),  // row 0
            (9, 0),  // row 0
            (10, 0), // row 0
            (0, 7),  // dy=7
            (1, 7),  // row 7
        ];
        for (dx, dy) in rows {
            assert_eq!(
                PassingDistanceCalc::for_deltas(dx, dy),
                Some(PassingDistance::LongPass),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Long Bomb ────────────────────────────────────────────────────────────

    #[test]
    fn long_bomb() {
        let rows = [
            (11, 0), // row 0
            (12, 0), // row 0
            (13, 0), // row 0
            (0, 11), // dy=11
            (1, 11), // row 11
            (0, 12), // dy=12
            (0, 13), // dy=13
            (1, 12), // row 12
            (2, 13), // row 13
        ];
        for (dx, dy) in rows {
            assert_eq!(
                PassingDistanceCalc::for_deltas(dx, dy),
                Some(PassingDistance::LongBomb),
                "dx={dx}, dy={dy}"
            );
        }
    }

    // ── Out of range ─────────────────────────────────────────────────────────

    #[test]
    fn negative_delta_returns_null() {
        assert_eq!(PassingDistanceCalc::for_deltas(-1, 0), None);
        assert_eq!(PassingDistanceCalc::for_deltas(0, -1), None);
    }

    #[test]
    fn delta_greater_than_13_returns_null() {
        assert_eq!(PassingDistanceCalc::for_deltas(14, 0), None);
        assert_eq!(PassingDistanceCalc::for_deltas(0, 14), None);
    }

    // ── Null cells in table (spaces) ─────────────────────────────────────────

    #[test]
    fn out_of_range_cells_return_null() {
        // dy=13, dx=3 → "B B B   " → index 3 is space → null
        assert_eq!(PassingDistanceCalc::for_deltas(3, 13), None);
        // dy=13, dx=13 → bottom-right corner is off the table edge → null
        assert_eq!(PassingDistanceCalc::for_deltas(13, 13), None);
    }

    // ── for_coordinates ──────────────────────────────────────────────────────

    #[test]
    fn for_coordinates_symmetrical() {
        // Passing from (5,7) to (8,7) = dx=3, dy=0 → QuickPass
        assert_eq!(
            PassingDistanceCalc::for_coordinates(5, 7, 8, 7),
            Some(PassingDistance::QuickPass)
        );
        // Symmetric: (8,7) to (5,7)
        assert_eq!(
            PassingDistanceCalc::for_coordinates(8, 7, 5, 7),
            Some(PassingDistance::QuickPass)
        );
    }

    #[test]
    fn for_coordinates_same_square_returns_null() {
        assert_eq!(PassingDistanceCalc::for_coordinates(5, 7, 5, 7), None);
    }

    #[test]
    fn for_coordinates_long_bomb_across_field() {
        // From x=1 to x=14 = dx=13, dy=0 → LongBomb
        assert_eq!(
            PassingDistanceCalc::for_coordinates(1, 7, 14, 7),
            Some(PassingDistance::LongBomb)
        );
    }
}
