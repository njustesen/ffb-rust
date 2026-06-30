use crate::enums::Direction;

/// 1:1 translation of com.fumbbl.ffb.factory.DirectionFactory.
pub struct DirectionFactory;

impl Default for DirectionFactory {
    fn default() -> Self { Self }
}

impl DirectionFactory {
    pub fn for_name(&self, name: &str) -> Option<Direction> {
        Direction::from_name(name)
    }

    /// Maps a d8 scatter roll to a Direction (Java DirectionFactory.forRoll).
    pub fn for_roll(&self, roll: i32) -> Option<Direction> {
        Direction::for_roll(roll)
    }

    pub fn transform(&self, directions: &[Direction]) -> Vec<Direction> {
        directions.iter().map(|d| d.transform()).collect()
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_roll_north() {
        assert_eq!(DirectionFactory.for_roll(1), Some(Direction::North));
    }

    #[test]
    fn for_roll_all() {
        let f = DirectionFactory;
        assert_eq!(f.for_roll(2), Some(Direction::Northeast));
        assert_eq!(f.for_roll(3), Some(Direction::East));
        assert_eq!(f.for_roll(4), Some(Direction::Southeast));
        assert_eq!(f.for_roll(5), Some(Direction::South));
        assert_eq!(f.for_roll(6), Some(Direction::Southwest));
        assert_eq!(f.for_roll(7), Some(Direction::West));
        assert_eq!(f.for_roll(8), Some(Direction::Northwest));
    }

    #[test]
    fn for_roll_out_of_range() {
        assert_eq!(DirectionFactory.for_roll(0), None);
        assert_eq!(DirectionFactory.for_roll(9), None);
    }

    #[test]
    fn transform_reverses_each_direction() {
        let f = DirectionFactory;
        let dirs = vec![Direction::North, Direction::South];
        let transformed = f.transform(&dirs);
        assert_eq!(transformed[0], Direction::North.transform());
        assert_eq!(transformed[1], Direction::South.transform());
    }
}
