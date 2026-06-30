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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_square_is_none() {
        // [0][0] is T (None)
        assert_eq!(PassingDistanceCalc::for_deltas(0, 0), None);
    }

    #[test]
    fn dx1_dy0_is_quick_pass() {
        // Row 0, col 1 = 'Q'
        assert_eq!(PassingDistanceCalc::for_deltas(1, 0), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn dx6_dy0_is_short_pass() {
        // Row 0: T Q Q Q S S S L ... → idx=6 is 'S'
        assert_eq!(PassingDistanceCalc::for_deltas(6, 0), Some(PassingDistance::ShortPass));
    }

    #[test]
    fn dx11_dy0_is_long_bomb() {
        // Row 0: ... B B B → idx=11 is 'B'
        assert_eq!(PassingDistanceCalc::for_deltas(11, 0), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn out_of_range_returns_none() {
        assert_eq!(PassingDistanceCalc::for_deltas(14, 0), None);
        assert_eq!(PassingDistanceCalc::for_deltas(0, 14), None);
        assert_eq!(PassingDistanceCalc::for_deltas(-1, 0), None);
    }

    #[test]
    fn for_coordinates_same_square_is_none() {
        assert_eq!(PassingDistanceCalc::for_coordinates(5, 5, 5, 5), None);
    }

    #[test]
    fn for_coordinates_adjacent_is_quick_pass() {
        // dx=1, dy=0 → QuickPass
        assert_eq!(PassingDistanceCalc::for_coordinates(5, 5, 6, 5), Some(PassingDistance::QuickPass));
    }

    #[test]
    fn for_coordinates_symmetric() {
        // from (0,0) to (3,3) should equal from (3,3) to (0,0)
        assert_eq!(
            PassingDistanceCalc::for_coordinates(0, 0, 3, 3),
            PassingDistanceCalc::for_coordinates(3, 3, 0, 0)
        );
    }

    #[test]
    fn bottom_right_corner_is_none() {
        // Row 13: "B B B" — dx >= 3 gives None
        assert_eq!(PassingDistanceCalc::for_deltas(13, 13), None);
    }

    #[test]
    fn dy13_dx0_is_long_bomb() {
        // Row 13: "B B B" → dx=0 is 'B'
        assert_eq!(PassingDistanceCalc::for_deltas(0, 13), Some(PassingDistance::LongBomb));
    }

    #[test]
    fn dx7_dy0_is_long_pass() {
        // Row 0: T Q Q Q S S S L → idx=7 is 'L'
        assert_eq!(PassingDistanceCalc::for_deltas(7, 0), Some(PassingDistance::LongPass));
    }
}
