use std::collections::HashSet;
use crate::types::{FieldCoordinate, FieldCoordinateBounds};
use crate::model::game::Game;

pub struct PathFinderExtension;

impl PathFinderExtension {
    pub fn new() -> Self {
        PathFinderExtension
    }

    pub fn has_prone_or_stunned_player_on_path(
        &self,
        game: &Game,
        from: FieldCoordinate,
        to: FieldCoordinate,
    ) -> bool {
        self.has_prone_or_stunned_players(game, self.find_possible_path_squares(from, to).into_iter())
    }

    pub fn has_prone_or_stunned_players(
        &self,
        game: &Game,
        coordinates: impl Iterator<Item = FieldCoordinate>,
    ) -> bool {
        coordinates
            .filter_map(|coord| game.field_model.player_at(coord))
            .filter_map(|id| game.field_model.player_state(id))
            .any(|state| state.is_stunned() || state.is_prone_or_stunned())
    }

    pub fn find_possible_path_squares(
        &self,
        from: FieldCoordinate,
        to: FieldCoordinate,
    ) -> HashSet<FieldCoordinate> {
        let x_variances = Self::dimension_variance(to.x - from.x);
        let y_variances = Self::dimension_variance(to.y - from.y);
        self.combine_variances(&x_variances, &y_variances, from)
            .into_iter()
            .filter(|c| FieldCoordinateBounds::FIELD.is_in_bounds(*c))
            .collect()
    }

    fn combine_variances(
        &self,
        x_variances: &[i32],
        y_variances: &[i32],
        start: FieldCoordinate,
    ) -> HashSet<FieldCoordinate> {
        let mut coordinates = HashSet::new();
        for &xv in x_variances {
            for &yv in y_variances {
                coordinates.insert(FieldCoordinate::new(start.x + xv, start.y + yv));
            }
        }
        coordinates
    }

    fn dimension_variance(diff: i32) -> Vec<i32> {
        match diff.abs() {
            2 => vec![diff / 2],
            1 => vec![diff, 0],
            0 => vec![0],
            _ => panic!("Received illegal dimension difference of {}", diff),
        }
    }
}

impl Default for PathFinderExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fc(x: i32, y: i32) -> FieldCoordinate {
        FieldCoordinate::new(x, y)
    }

    #[test]
    fn dimension_variance_zero_returns_zero() {
        assert_eq!(PathFinderExtension::dimension_variance(0), vec![0]);
    }

    #[test]
    fn dimension_variance_one_returns_diff_and_zero() {
        assert_eq!(PathFinderExtension::dimension_variance(1), vec![1, 0]);
        assert_eq!(PathFinderExtension::dimension_variance(-1), vec![-1, 0]);
    }

    #[test]
    fn dimension_variance_two_returns_half() {
        assert_eq!(PathFinderExtension::dimension_variance(2), vec![1]);
        assert_eq!(PathFinderExtension::dimension_variance(-2), vec![-1]);
    }

    #[test]
    fn find_possible_path_squares_diagonal_jump() {
        let ext = PathFinderExtension::new();
        // Jump from (5,5) to (7,7): delta=(2,2) → each variance is [1], so only intermediate is (6,6)
        let squares = ext.find_possible_path_squares(fc(5, 5), fc(7, 7));
        assert!(squares.contains(&fc(6, 6)));
        assert_eq!(squares.len(), 1);
    }

    #[test]
    fn find_possible_path_squares_orthogonal_jump() {
        let ext = PathFinderExtension::new();
        // Jump from (5,5) to (7,5): delta=(2,0) → x_variance=[1], y_variance=[0] → intermediate (6,5)
        let squares = ext.find_possible_path_squares(fc(5, 5), fc(7, 5));
        assert!(squares.contains(&fc(6, 5)));
        assert_eq!(squares.len(), 1);
    }

    #[test]
    fn find_possible_path_squares_filters_out_of_bounds() {
        let ext = PathFinderExtension::new();
        // Jump from (0,0) to (2,0): intermediate is (1,0) - in bounds
        let squares = ext.find_possible_path_squares(fc(0, 0), fc(2, 0));
        for sq in &squares {
            assert!(FieldCoordinateBounds::FIELD.is_in_bounds(*sq));
        }
    }

    #[test]
    fn find_possible_path_squares_adjacent_delta_1() {
        let ext = PathFinderExtension::new();
        // Jump from (5,5) to (6,7): delta=(1,2) → x_var=[1,0], y_var=[1] → coords (6,6) and (5,6)
        let squares = ext.find_possible_path_squares(fc(5, 5), fc(6, 7));
        assert!(squares.contains(&fc(6, 6)));
        assert!(squares.contains(&fc(5, 6)));
    }

    #[test]
    fn new_creates_instance() {
        let _ext = PathFinderExtension::new();
        let _ext2 = PathFinderExtension::default();
    }
}
