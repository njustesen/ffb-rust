use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.model.Sketch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sketch {
    pub positions: Vec<FieldCoordinate>,
}

impl Sketch {
    pub fn new() -> Self { Self::default() }
    pub fn add_position(&mut self, pos: FieldCoordinate) { self.positions.push(pos); }
    pub fn len(&self) -> usize { self.positions.len() }
    pub fn is_empty(&self) -> bool { self.positions.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(Sketch::new().is_empty());
    }

    #[test]
    fn add_position_increases_len() {
        let mut s = Sketch::new();
        s.add_position(FieldCoordinate::new(0, 0));
        assert_eq!(s.len(), 1);
    }
}
