use std::collections::HashSet;
use crate::types::{FieldCoordinate, FIELD_WIDTH, FIELD_HEIGHT};

/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFinderExtension`.
#[derive(Debug, Default)]
pub struct PathFinderExtension;

impl PathFinderExtension {
    pub fn new() -> Self { Self }

    /// Java: `findPossiblePathSquares(from, to)` — all intermediate squares between two coordinates.
    pub fn find_possible_path_squares(
        &self,
        from: FieldCoordinate,
        to: FieldCoordinate,
    ) -> HashSet<FieldCoordinate> {
        let x_variances = Self::dimension_variance(to.x - from.x);
        let y_variances = Self::dimension_variance(to.y - from.y);
        let mut result = HashSet::new();
        for dx in &x_variances {
            for dy in &y_variances {
                let c = FieldCoordinate::new(from.x + dx, from.y + dy);
                if c.x >= 0 && c.x < FIELD_WIDTH && c.y >= 0 && c.y < FIELD_HEIGHT {
                    result.insert(c);
                }
            }
        }
        result
    }

    fn dimension_variance(delta: i32) -> Vec<i32> {
        if delta == 0 { return vec![0]; }
        let abs = delta.abs();
        (0..=abs).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_path_has_intermediate_squares() {
        let ext = PathFinderExtension::new();
        let squares = ext.find_possible_path_squares(
            FieldCoordinate::new(0, 0),
            FieldCoordinate::new(3, 0),
        );
        // Should include (0,0), (1,0), (2,0), (3,0)
        assert!(squares.contains(&FieldCoordinate::new(1, 0)));
        assert!(squares.contains(&FieldCoordinate::new(2, 0)));
    }

    #[test]
    fn same_square_returns_single() {
        let ext = PathFinderExtension::new();
        let squares = ext.find_possible_path_squares(
            FieldCoordinate::new(5, 5),
            FieldCoordinate::new(5, 5),
        );
        assert_eq!(squares.len(), 1);
    }
}
